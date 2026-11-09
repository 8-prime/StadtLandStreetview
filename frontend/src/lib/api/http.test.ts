import { afterEach, expect, it, vi } from 'vitest';
import { httpApi } from './http';

class FakeSocket {
  static OPEN = 1;
  static current: FakeSocket;
  readyState = 1;
  onopen?: () => void;
  onclose?: () => void;
  onmessage?: (event: { data: string }) => void;
  onerror?: () => void;
  send = vi.fn();
  close = vi.fn();
  constructor() { FakeSocket.current = this; }
}

afterEach(() => { vi.unstubAllGlobals(); });

it('handles proximity messages without fetching game snapshots and reports only position/step', async () => {
  const fetch = vi.fn().mockResolvedValue({ ok: true, status: 200, json: async () => ({ revision: 1 }) });
  vi.stubGlobal('fetch', fetch);
  vi.stubGlobal('WebSocket', FakeSocket);
  vi.stubGlobal('location', { href: 'http://localhost/' });
  const proximity = vi.fn();
  const territory = vi.fn();
  const unsubscribe = httpApi.subscribe('ROOM', vi.fn(), vi.fn(), vi.fn(), proximity, territory);
  try {
    await vi.waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
    const socket = FakeSocket.current;
    socket.onmessage?.({ data: JSON.stringify({ type: 'proximity', stepId: 'search-step', band: 'nearby' }) });
    expect(proximity).toHaveBeenCalledWith({ stepId: 'search-step', band: 'nearby' });
    socket.onmessage?.({ data: JSON.stringify({ type: 'territory', stepId: 'search-step', banned: 'DE' }) });
    socket.onmessage?.({ data: JSON.stringify({ type: 'territory', stepId: 'search-step', banned: null }) });
    expect(territory.mock.calls).toEqual([[{ stepId: 'search-step', banned: 'DE' }], [{ stepId: 'search-step', banned: null }]]);
    expect(fetch).toHaveBeenCalledTimes(1);
    httpApi.reportPosition('ROOM', { stepId: 'search-step', position: { lat: 52.52, lng: 13.405 } });
    expect(JSON.parse(socket.send.mock.calls[0][0])).toEqual({ type: 'position', stepId: 'search-step', position: { lat: 52.52, lng: 13.405 } });
    socket.onmessage?.({ data: JSON.stringify({ type: 'state-changed', revision: 2 }) });
    await vi.waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
  } finally { unsubscribe(); }
});

it('sends lobby rule changes as a partial settings patch', async () => {
  const fetch = vi.fn().mockResolvedValue({ ok: true, status: 204 });
  vi.stubGlobal('fetch', fetch);
  await httpApi.send('ROOM', { type: 'update-settings', durationSeconds: 5400, bannedCountries: ['DE'] });
  const [url, init] = fetch.mock.calls[0];
  expect(url).toBe('/api/sessions/ROOM/settings');
  expect(init.method).toBe('PATCH');
  expect(JSON.parse(init.body)).toEqual({ durationSeconds: 5400, bannedCountries: ['DE'] });
});
