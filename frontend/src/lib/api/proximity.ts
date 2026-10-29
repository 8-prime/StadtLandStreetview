import type { ProximityBand, Viewpoint } from './types';

type Position = Viewpoint['position'];
export const presenceLifetime = 15_000;

export function distanceMeters(a: Position, b: Position) {
  const radians = Math.PI / 180;
  const lat = Math.sin((b.lat - a.lat) * radians / 2);
  const lng = Math.sin((b.lng - a.lng) * radians / 2);
  const haversine = lat * lat + Math.cos(a.lat * radians) * Math.cos(b.lat * radians) * lng * lng;
  return 2 * 6_371_000 * Math.asin(Math.sqrt(Math.min(1, Math.max(0, haversine))));
}

export function proximityBand(position: Position | null, others: Position[]): ProximityBand {
  if (!position) return 'inactive';
  const closest = others.reduce((minimum, other) => Math.min(minimum, distanceMeters(position, other)), Infinity);
  return closest <= 250 ? 'close' : closest <= 5000 ? 'nearby' : 'quiet';
}
