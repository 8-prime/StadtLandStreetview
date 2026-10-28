import { mockApi } from './mock';
import { httpApi } from './http';

export const mockMode = import.meta.env.VITE_API_MODE !== 'http';
export const api = mockMode ? mockApi : httpApi;
