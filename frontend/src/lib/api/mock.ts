import { applyAction, createRoom, reconcile, simulatePeers, snapshot, tick, uid, type Room } from './engine';
import type { SessionApi } from './types';

const roomKey = (id: string) => `sls:room:${id}`;
const identityKey = (id: string) => `sls:identity:${id}`;
const read = (id: string): Room => {
  const value = localStorage.getItem(roomKey(id));
  if (!value) throw new Error('Room not found. Local rooms work only in this browser; try creating a new one.');
  return JSON.parse(value);
};
const identity = (id: string): string => {
  const value = sessionStorage.getItem(identityKey(id));
  if (!value) throw new Error('Join this room to continue.');
  return value;
};
const save = (room: Room) => {
  room.revision++;
  localStorage.setItem(roomKey(room.id), JSON.stringify(room));
  window.dispatchEvent(new CustomEvent('sls:changed', { detail: room.id }));
};

// Web Locks serialize mutations between tabs. This is a prototype store, not a backend.
async function update(id: string, action: (room: Room) => void) {
  const change = () => {
    const room = read(id);
    action(room);
    save(room);
  };
  if (navigator.locks) await navigator.locks.request(`sls:${id}`, change);
  else change();
}

export const mockApi: SessionApi = {
  async create(name, durationSeconds, demo = false) {
    let id: string;
    do { id = uid().replaceAll('-', '').slice(0, 6).toUpperCase(); } while (localStorage.getItem(roomKey(id)));
    const playerId = uid();
    const room = createRoom(id, name.trim().slice(0, 24), playerId, durationSeconds, demo);
    sessionStorage.setItem(identityKey(id), playerId);
    save(room);
    return id;
  },
  async join(id, name) {
    const existing = sessionStorage.getItem(identityKey(id));
    if (existing && read(id).players.some(p => p.id === existing && !p.left)) return;
    const playerId = uid();
    await update(id, room => {
      if (room.demo) throw new Error('Solo demos have simulated peers. Create a local room to play across tabs.');
      if (!['lobby', 'categories'].includes(room.phase)) throw new Error('This round has already started.');
      if (room.players.filter(p => !p.left).length >= 12) throw new Error('This room is full (12 players).');
      room.players.push({ id: playerId, name: name.trim().slice(0, 24), ready: false, connected: true, color: room.players.length % 6, score: 0, left: false, bot: false, lastSeen: Date.now() });
      room.players.forEach(p => p.ready = false);
      room.stepId = uid();
    });
    sessionStorage.setItem(identityKey(id), playerId);
  },
  async get(id) { return snapshot(read(id), identity(id)); },
  async send(id, action) { await update(id, room => applyAction(room, identity(id), action)); },
  async leave(id) {
    const playerId = identity(id);
    await update(id, room => {
      const player = room.players.find(p => p.id === playerId);
      if (player) { player.left = true; player.connected = false; player.ready = false; }
      delete room.votes[playerId];
      reconcile(room);
    });
    sessionStorage.removeItem(identityKey(id));
  },
  subscribe(id, onState, onConnection, onError) {
    let disposed = false;
    let lastRevision = -1;
    const emit = () => {
      if (disposed) return;
      try {
        const room = read(id);
        if (room.revision < lastRevision) return;
        lastRevision = room.revision;
        onState(snapshot(room, identity(id)));
        onConnection('connected');
      } catch (error) { onConnection('offline'); onError((error as Error).message); }
    };
    const changed = (event: Event) => {
      if (event instanceof StorageEvent ? event.key === roomKey(id) : (event as CustomEvent).detail === id) emit();
    };
    const maintain = () => update(id, room => {
      const now = Date.now();
      const player = room.players.find(p => p.id === identity(id) && !p.left);
      if (player) { player.lastSeen = now; player.connected = true; }
      room.players.forEach(p => { if (!p.bot && p.id !== player?.id) p.connected = !p.left && now - p.lastSeen < 45000; });
      tick(room, now);
      simulatePeers(room, now);
    }).catch(error => { if (!disposed) onError((error as Error).message); });
    window.addEventListener('storage', changed);
    window.addEventListener('sls:changed', changed);
    const timer = window.setInterval(maintain, 1500);
    onConnection('connecting');
    emit();
    void maintain();
    return () => {
      disposed = true;
      clearInterval(timer);
      window.removeEventListener('storage', changed);
      window.removeEventListener('sls:changed', changed);
    };
  },
};
