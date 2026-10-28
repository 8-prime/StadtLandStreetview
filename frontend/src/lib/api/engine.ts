import type { Action, Category, Finding, Phase, Player, ReviewResult, SessionState, Vote } from './types';

export interface Room {
  id: string;
  revision: number;
  stepId: string;
  phase: Phase;
  demo: boolean;
  durationSeconds: number;
  searchEndsAt: number | null;
  players: (Player & { bot: boolean; lastSeen: number })[];
  categories: Category[];
  findings: Finding[];
  selections: Record<string, Record<string, string>>;
  queue: string[];
  reviewIndex: number;
  revealed: boolean;
  votes: Record<string, Vote>;
  results: ReviewResult[];
}

export const uid = () => crypto.randomUUID();
export const demoView = (scene = 0) => ({ panoramaId: `demo-${scene}`, position: { lat: 52.52, lng: 13.405 }, heading: 90, pitch: 0, zoom: 1, demoScene: scene });

export function createRoom(id: string, name: string, playerId: string, durationSeconds: number, demo: boolean): Room {
  const names = demo ? [name, 'Mika', 'Jules'] : [name];
  return {
    id, revision: 0, stepId: uid(), phase: 'lobby', demo, durationSeconds,
    searchEndsAt: null, categories: [], findings: [], selections: {}, queue: [],
    reviewIndex: 0, revealed: false, votes: {}, results: [],
    players: names.map((n, i) => ({ id: i === 0 ? playerId : uid(), name: n, ready: false, connected: true, color: i, score: 0, left: false, bot: i > 0, lastSeen: Date.now() })),
  };
}

function newStep(room: Room, phase: Phase) {
  room.phase = phase;
  room.stepId = uid();
  room.players.forEach(p => p.ready = false);
}

export function currentFinding(room: Room) {
  return room.findings.find(f => f.id === room.queue[room.reviewIndex]);
}

function autoSelections(room: Room) {
  for (const player of room.players) {
    const selected = room.selections[player.id] ??= {};
    for (const category of room.categories) {
      const candidates = room.findings.filter(f => f.playerId === player.id && f.categoryId === category.id);
      if (!candidates.some(f => f.id === selected[category.id])) {
        delete selected[category.id];
        if (candidates[0]) selected[category.id] = candidates[0].id;
      }
    }
  }
}

export function tick(room: Room, now: number): boolean {
  if (room.phase !== 'searching' || room.searchEndsAt === null || now < room.searchEndsAt) return false;
  autoSelections(room);
  newStep(room, 'selection');
  return true;
}

function advance(room: Room, now: number) {
  const active = room.players.filter(p => !p.left);
  if (!active.length || !active.every(p => p.ready)) return;
  if (room.phase === 'lobby') {
    newStep(room, 'categories');
    if (room.demo) room.categories = ['A person walking a dog', 'A car parked illegally', 'Something that makes you smile'].map(text => ({ id: uid(), text }));
  } else if (room.phase === 'categories') {
    if (!room.categories.length) throw new Error('Add at least one category before starting.');
    newStep(room, 'searching');
    room.searchEndsAt = now + room.durationSeconds * 1000;
    if (room.demo) {
      room.players.filter(p => p.bot).forEach((p, i) => room.categories.forEach((c, j) => {
        room.findings.push({ id: uid(), categoryId: c.id, playerId: p.id, view: demoView((i + j + 1) % 3), createdAt: now });
      }));
    }
  } else if (room.phase === 'selection') {
    autoSelections(room);
    room.queue = room.categories.flatMap(c => room.players.flatMap(p => {
      const id = room.selections[p.id]?.[c.id];
      return id ? [id] : [];
    }));
    room.reviewIndex = 0;
    room.revealed = false;
    room.votes = {};
    newStep(room, room.queue.length ? 'review' : 'finished');
  } else if (room.phase === 'review') {
    if (!room.revealed) {
      const finding = currentFinding(room)!;
      const accepts = Object.values(room.votes).filter(v => v === 'accept').length;
      const rejects = Object.values(room.votes).filter(v => v === 'reject').length;
      const accepted = accepts > rejects;
      room.results.push({ itemId: finding.id, playerId: finding.playerId, categoryId: finding.categoryId, accepted, accepts, rejects });
      if (accepted) room.players.find(p => p.id === finding.playerId)!.score++;
      room.revealed = true;
      newStep(room, 'review');
    } else {
      room.reviewIndex++;
      room.revealed = false;
      room.votes = {};
      newStep(room, room.reviewIndex < room.queue.length ? 'review' : 'finished');
    }
  }
}

export function applyAction(room: Room, playerId: string, action: Action, now = Date.now()) {
  const player = room.players.find(p => p.id === playerId && !p.left);
  if (!player) throw new Error('You are no longer in this room. Join again to continue.');
  tick(room, now);
  if (action.type === 'ready') {
    if (room.stepId !== action.stepId) throw new Error('The group has moved on. Please check the current step.');
    if (room.phase === 'searching' || room.phase === 'finished') throw new Error('Readiness is not used in this phase.');
    if (action.ready && room.phase === 'categories' && !room.categories.length) throw new Error('Add at least one category first.');
    const finding = currentFinding(room);
    if (action.ready && room.phase === 'review' && !room.revealed && finding?.playerId !== playerId && !room.votes[playerId]) throw new Error('Cast your vote before marking yourself ready.');
    player.ready = action.ready;
  } else if (action.type === 'add-category' || action.type === 'edit-category' || action.type === 'delete-category') {
    if (room.phase !== 'categories') throw new Error('Categories are locked for this round.');
    if (action.type !== 'delete-category' && (!action.text.trim() || action.text.trim().length > 100)) throw new Error('Use between 1 and 100 characters.');
    if (action.type === 'add-category') {
      if (room.categories.length >= 12) throw new Error('This round can have up to 12 categories.');
      room.categories.push({ id: uid(), text: action.text.trim() });
    } else {
      const category = room.categories.find(c => c.id === action.categoryId);
      if (!category) throw new Error('That category no longer exists.');
      if (action.type === 'edit-category') category.text = action.text.trim();
      else room.categories = room.categories.filter(c => c.id !== category.id);
    }
    newStep(room, 'categories');
  } else if (action.type === 'save-finding') {
    if (room.phase !== 'searching') throw new Error('Search time is over. Choose from your saved views.');
    if (!room.categories.some(c => c.id === action.categoryId)) throw new Error('Unknown category.');
    if (action.view.demoScene !== undefined && !room.demo) throw new Error('Only real Street View findings can be saved in this room.');
    const candidates = room.findings.filter(f => f.playerId === playerId && f.categoryId === action.categoryId);
    if (candidates.length >= 12) throw new Error('You have 12 candidates here. Remove one before adding another.');
    room.findings.push({ id: uid(), playerId, categoryId: action.categoryId, view: structuredClone(action.view), createdAt: now });
    autoSelections(room);
  } else if (action.type === 'delete-finding') {
    if (!['searching', 'selection'].includes(room.phase)) throw new Error('Findings are locked.');
    const finding = room.findings.find(f => f.id === action.findingId && f.playerId === playerId);
    if (!finding) throw new Error('Finding not found.');
    room.findings = room.findings.filter(f => f.id !== finding.id);
    autoSelections(room);
    player.ready = false;
  } else if (action.type === 'select') {
    if (!['searching', 'selection'].includes(room.phase)) throw new Error('Selections are locked.');
    if (!room.findings.some(f => f.id === action.findingId && f.categoryId === action.categoryId && f.playerId === playerId)) throw new Error('Invalid final selection.');
    (room.selections[playerId] ??= {})[action.categoryId] = action.findingId;
    player.ready = false;
  } else if (action.type === 'vote') {
    const finding = currentFinding(room);
    if (room.phase !== 'review' || room.revealed || finding?.id !== action.itemId) throw new Error('Voting has closed for that finding.');
    if (finding.playerId === playerId) throw new Error('You cannot vote on your own finding.');
    room.votes[playerId] = action.vote;
    player.ready = false;
  } else if (action.type === 'demo-end-search') {
    if (!room.demo || room.phase !== 'searching') throw new Error('This shortcut is only available in the solo demo.');
    room.searchEndsAt = now;
    tick(room, now);
  }
  advance(room, now);
}

export function simulatePeers(room: Room, now = Date.now()): boolean {
  if (!room.demo || room.phase === 'searching' || room.phase === 'finished') return false;
  const bots = room.players.filter(p => p.bot && !p.ready);
  if (!bots.length || (room.phase === 'categories' && !room.categories.length)) return false;
  for (const bot of bots) {
    const finding = currentFinding(room);
    if (room.phase === 'review' && !room.revealed && finding && finding.playerId !== bot.id) {
      room.votes[bot.id] = (room.reviewIndex + bot.color) % 4 === 0 ? 'reject' : 'accept';
    }
    bot.ready = true;
  }
  advance(room, now);
  return true;
}

export function snapshot(room: Room, meId: string): SessionState {
  if (!room.players.some(p => p.id === meId && !p.left)) throw new Error('Join this room to continue.');
  const finding = currentFinding(room);
  return structuredClone({
    id: room.id, revision: room.revision, stepId: room.stepId, phase: room.phase, meId,
    demo: room.demo, durationSeconds: room.durationSeconds, serverTime: Date.now(), searchEndsAt: room.searchEndsAt,
    players: room.players.map(({ bot: _bot, lastSeen: _seen, ...p }) => p), categories: room.categories,
    ownFindings: room.findings.filter(f => f.playerId === meId), selections: room.selections[meId] ?? {},
    review: room.phase === 'review' && finding ? {
      id: finding.id, finding, index: room.reviewIndex, total: room.queue.length,
      revealed: room.revealed, ownVote: room.votes[meId] ?? null,
      votesCast: Object.keys(room.votes).length, eligibleVoters: room.players.filter(p => p.id !== finding.playerId && !p.left).length,
      result: room.revealed ? room.results.find(r => r.itemId === finding.id) ?? null : null,
    } : null,
    results: room.results,
  });
}

export function reconcile(room: Room) { advance(room, Date.now()); }
