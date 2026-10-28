import { describe, expect, it } from 'vitest';
import { applyAction, createRoom, currentFinding, demoView, reconcile, snapshot, tick, type Room } from './engine';

function room() {
  const r = createRoom('TEST12', 'Alex', 'alex', 60, true);
  r.players.forEach(p => p.bot = false);
  return r;
}
function readyAll(r: Room) {
  const stepId = r.stepId;
  for (const p of r.players.filter(p => !p.left)) applyAction(r, p.id, { type: 'ready', stepId, ready: true }, 1000);
}
function searching() {
  const r = room();
  readyAll(r);
  readyAll(r);
  for (const player of r.players.slice(1)) {
    for (const category of r.categories) {
      applyAction(r, player.id, { type: 'save-finding', categoryId: category.id, view: demoView(player.color) }, 2000);
    }
  }
  return r;
}

describe('peer progression and findings', () => {
  it('allows undo, waits for all peers, resets readiness, and rejects stale acknowledgements', () => {
    const r = room();
    const oldStep = r.stepId;
    applyAction(r, 'alex', { type: 'ready', stepId: oldStep, ready: true });
    expect(r.phase).toBe('lobby');
    applyAction(r, 'alex', { type: 'ready', stepId: oldStep, ready: false });
    expect(r.players[0].ready).toBe(false);
    readyAll(r);
    expect(r.phase).toBe('categories');
    expect(r.players.every(p => !p.ready)).toBe(true);
    expect(() => applyAction(r, 'alex', { type: 'ready', stepId: oldStep, ready: true })).toThrow('moved on');
  });

  it('invalidates category agreement after an edit and prevents an empty search', () => {
    const r = room();
    readyAll(r);
    applyAction(r, 'alex', { type: 'ready', stepId: r.stepId, ready: true });
    applyAction(r, 'alex', { type: 'edit-category', categoryId: r.categories[0].id, text: 'A blue door' });
    expect(r.players.every(p => !p.ready)).toBe(true);
    r.categories = [];
    expect(() => applyAction(r, 'alex', { type: 'ready', stepId: r.stepId, ready: true })).toThrow('category');
  });

  it('expires on one overall deadline, rejects late findings, and cannot end early via readiness', () => {
    const r = searching();
    expect(() => applyAction(r, 'alex', { type: 'ready', stepId: r.stepId, ready: true }, 2000)).toThrow('not used');
    expect(tick(r, 60999)).toBe(false);
    expect(tick(r, 61000)).toBe(true);
    expect(r.phase).toBe('selection');
    expect(() => applyAction(r, 'alex', { type: 'save-finding', categoryId: r.categories[0].id, view: demoView() }, 62000)).toThrow('over');
  });

  it('defaults to the first candidate, handles missing findings implicitly, and repairs deleted selections', () => {
    const r = searching();
    const categoryId = r.categories[0].id;
    applyAction(r, 'alex', { type: 'save-finding', categoryId, view: demoView() }, 2000);
    applyAction(r, 'alex', { type: 'save-finding', categoryId, view: demoView(1) }, 3000);
    const candidates = r.findings.filter(f => f.playerId === 'alex');
    expect(r.selections.alex[categoryId]).toBe(candidates[0].id);
    expect(r.selections.alex[r.categories[1].id]).toBeUndefined();
    tick(r, 61000);
    applyAction(r, 'alex', { type: 'ready', stepId: r.stepId, ready: true }, 62000);
    applyAction(r, 'alex', { type: 'select', categoryId, findingId: candidates[1].id }, 62000);
    expect(r.players[0].ready).toBe(false);
    applyAction(r, 'alex', { type: 'delete-finding', findingId: candidates[1].id }, 62000);
    expect(r.selections.alex[categoryId]).toBe(candidates[0].id);
    applyAction(r, 'alex', { type: 'delete-finding', findingId: candidates[0].id }, 62000);
    expect(r.selections.alex[categoryId]).toBeUndefined();
  });

  it('never includes another player’s candidates or private votes in a snapshot', () => {
    const r = searching();
    expect(r.findings.length).toBeGreaterThan(0);
    expect(snapshot(r, 'alex').ownFindings).toEqual([]);
    tick(r, 61000);
    readyAll(r);
    const finding = currentFinding(r)!;
    applyAction(r, 'alex', { type: 'vote', itemId: finding.id, vote: 'reject' }, 62000);
    const other = snapshot(r, r.players[1].id);
    expect(other.review?.ownVote).toBeNull();
    expect(other.review?.result).toBeNull();
    expect(other).not.toHaveProperty('votes');
  });

  it('orders review by category then player and skips absent submissions', () => {
    const r = searching();
    applyAction(r, 'alex', { type: 'save-finding', categoryId: r.categories[0].id, view: demoView() }, 2000);
    tick(r, 61000);
    readyAll(r);
    expect(r.queue.map(id => r.findings.find(f => f.id === id)!.categoryId)).toEqual([
      r.categories[0].id, r.categories[0].id, r.categories[0].id,
      r.categories[1].id, r.categories[1].id,
      r.categories[2].id, r.categories[2].id,
    ]);
    expect(currentFinding(r)?.playerId).toBe('alex');
  });

  it('requires a vote before readiness, forbids self-voting, treats ties as failure, and completes the round', () => {
    const r = searching();
    r.categories = [r.categories[0]];
    tick(r, 61000);
    readyAll(r);
    const finding = currentFinding(r)!;
    expect(() => applyAction(r, finding.playerId, { type: 'vote', itemId: finding.id, vote: 'accept' })).toThrow('own');
    expect(() => applyAction(r, 'alex', { type: 'ready', stepId: r.stepId, ready: true })).toThrow('Cast your vote');
    for (let i = 0; i < 2; i++) {
      const current = currentFinding(r)!;
      const voters = r.players.filter(p => p.id !== current.playerId);
      voters.forEach((p, n) => applyAction(r, p.id, { type: 'vote', itemId: current.id, vote: n === 0 ? 'accept' : 'reject' }));
      readyAll(r);
      expect(r.revealed).toBe(true);
      expect(r.results.at(-1)?.accepted).toBe(false);
      expect(() => applyAction(r, 'alex', { type: 'vote', itemId: current.id, vote: 'accept' })).toThrow('closed');
      readyAll(r);
    }
    expect(r.phase).toBe('finished');
    expect(r.results).toHaveLength(2);
    expect(r.players.every(p => p.score === 0)).toBe(true);
  });

  it('keeps disconnected peers in readiness but excludes explicit departures', () => {
    const r = room();
    r.players[2].connected = false;
    r.players[0].ready = r.players[1].ready = true;
    reconcile(r);
    expect(r.phase).toBe('lobby');
    r.players[2].left = true;
    reconcile(r);
    expect(r.phase).toBe('categories');
  });

  it('finishes gracefully if nobody has a finding', () => {
    const r = searching();
    r.findings = [];
    tick(r, 61000);
    readyAll(r);
    expect(r.phase).toBe('finished');
    expect(r.results).toEqual([]);
  });
});
