import type { Action, SessionApi, SessionState } from './types';

async function request<T>(path: string, method = 'GET', body?: unknown): Promise<T> {
  const response = await fetch(`/api${path}`, {
    method, credentials: 'same-origin',
    headers: body === undefined ? {} : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => null);
    throw new Error(error?.message || error?.error || `Request failed (${response.status}). Please try again.`);
  }
  return response.status === 204 ? undefined as T : response.json();
}

function actionRequest(id: string, action: Action) {
  const base = `/sessions/${encodeURIComponent(id)}`;
  switch (action.type) {
    case 'ready': return request(`${base}/ready`, 'PUT', { stepId: action.stepId, ready: action.ready });
    case 'add-category': return request(`${base}/categories`, 'POST', { text: action.text });
    case 'edit-category': return request(`${base}/categories/${encodeURIComponent(action.categoryId)}`, 'PATCH', { text: action.text });
    case 'delete-category': return request(`${base}/categories/${encodeURIComponent(action.categoryId)}`, 'DELETE');
    case 'save-finding': {
      if (action.view.demoScene !== undefined) throw new Error('Illustrated demo views cannot be submitted to the server.');
      return request(`${base}/findings`, 'POST', { categoryId: action.categoryId, view: action.view });
    }
    case 'delete-finding': return request(`${base}/findings/${encodeURIComponent(action.findingId)}`, 'DELETE');
    case 'select': return request(`${base}/selections/${encodeURIComponent(action.categoryId)}`, 'PUT', { findingId: action.findingId });
    case 'vote': return request(`${base}/review/${encodeURIComponent(action.itemId)}/vote`, 'PUT', { vote: action.vote });
    case 'demo-end-search': throw new Error('Demo shortcuts are not available on the server.');
  }
}

export const httpApi: SessionApi = {
  async create(name, durationSeconds, demo) {
    if (demo) throw new Error('Solo demos are available only in mock mode.');
    const { id } = await request<{ id: string }>('/sessions', 'POST', { name, durationSeconds });
    return id;
  },
  async join(id, name) { await request(`/sessions/${encodeURIComponent(id)}/join`, 'POST', { name }); },
  get(id) { return request<SessionState>(`/sessions/${encodeURIComponent(id)}`); },
  async send(id, action) { await actionRequest(id, action); },
  async leave(id) { await request(`/sessions/${encodeURIComponent(id)}/leave`, 'POST'); },
  subscribe(id, onState, onConnection, onError) {
    let disposed = false;
    let socket: WebSocket | undefined;
    let retry: ReturnType<typeof setTimeout>;
    let attempt = 0;
    let newestRevision = -1;
    const refresh = async () => {
      try {
        const state = await httpApi.get(id);
        if (!disposed && state.revision >= newestRevision) {
          newestRevision = state.revision;
          onState(state);
        }
      } catch (error) { if (!disposed) onError((error as Error).message); }
    };
    const connect = () => {
      if (disposed) return;
      onConnection(attempt ? 'reconnecting' : 'connecting');
      const url = new URL(`/api/sessions/${encodeURIComponent(id)}/ws`, location.href);
      url.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const current = new WebSocket(url);
      socket = current;
      current.onopen = () => { attempt = 0; onConnection('connected'); void refresh(); };
      current.onmessage = () => { void refresh(); };
      current.onerror = () => current.close();
      current.onclose = () => {
        if (disposed) return;
        onConnection('reconnecting');
        retry = setTimeout(connect, Math.min(1000 * 2 ** attempt++, 15000));
      };
    };
    void refresh();
    connect();
    return () => { disposed = true; clearTimeout(retry); socket?.close(); };
  },
};
