export type Phase = 'lobby' | 'categories' | 'searching' | 'selection' | 'review' | 'finished';
export type Vote = 'accept' | 'reject';
export type Connection = 'connecting' | 'connected' | 'reconnecting' | 'offline';

export interface Viewpoint {
  panoramaId: string;
  position: { lat: number; lng: number };
  heading: number;
  pitch: number;
  zoom: number;
  // Illustrated scenes exist only in the local demo, never in HTTP submissions.
  demoScene?: number;
}

export interface Player {
  id: string;
  name: string;
  ready: boolean;
  connected: boolean;
  color: number;
  score: number;
  left: boolean;
}

export interface Category { id: string; text: string }
export interface Finding {
  id: string;
  playerId: string;
  categoryId: string;
  view: Viewpoint;
  createdAt: number;
}
export interface ReviewResult {
  itemId: string;
  categoryId: string;
  playerId: string;
  accepted: boolean;
  accepts: number;
  rejects: number;
}
export interface ReviewItem {
  id: string;
  finding: Finding;
  index: number;
  total: number;
  revealed: boolean;
  ownVote: Vote | null;
  votesCast: number;
  eligibleVoters: number;
  result: ReviewResult | null;
}

export interface SessionState {
  id: string;
  revision: number;
  stepId: string;
  phase: Phase;
  meId: string;
  demo: boolean;
  durationSeconds: number;
  serverTime: number;
  searchEndsAt: number | null;
  players: Player[];
  categories: Category[];
  ownFindings: Finding[];
  selections: Record<string, string>;
  review: ReviewItem | null;
  results: ReviewResult[];
}

export type Action =
  | { type: 'ready'; stepId: string; ready: boolean }
  | { type: 'add-category'; text: string }
  | { type: 'edit-category'; categoryId: string; text: string }
  | { type: 'delete-category'; categoryId: string }
  | { type: 'save-finding'; categoryId: string; view: Viewpoint }
  | { type: 'delete-finding'; findingId: string }
  | { type: 'select'; categoryId: string; findingId: string }
  | { type: 'vote'; itemId: string; vote: Vote }
  | { type: 'demo-end-search' };

export interface SessionApi {
  create(name: string, durationSeconds: number, demo?: boolean): Promise<string>;
  join(id: string, name: string): Promise<void>;
  get(id: string): Promise<SessionState>;
  send(id: string, action: Action): Promise<void>;
  leave(id: string): Promise<void>;
  subscribe(id: string, state: (value: SessionState) => void, connection: (value: Connection) => void, error: (message: string) => void): () => void;
}
