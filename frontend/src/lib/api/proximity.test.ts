import { describe, expect, it } from 'vitest';
import { distanceMeters, proximityBand } from './proximity';

describe('neighbourhood radio distances', () => {
  const origin = { lat: 0, lng: 0 };
  const north = (meters: number) => ({ lat: meters / 6_371_000 * 180 / Math.PI, lng: 0 });

  it('uses the nearest peer with 5 km and 250 m boundaries', () => {
    expect(proximityBand(null, [origin])).toBe('inactive');
    expect(proximityBand(origin, [])).toBe('quiet');
    expect(proximityBand(origin, [north(5001)])).toBe('quiet');
    expect(proximityBand(origin, [north(4999)])).toBe('nearby');
    expect(proximityBand(origin, [north(251)])).toBe('nearby');
    expect(proximityBand(origin, [north(249), north(9000)])).toBe('close');
    expect(proximityBand(origin, [origin])).toBe('close');
  });

  it('handles the date line, poles, and opposite sides of Earth', () => {
    expect(distanceMeters({ lat: 0, lng: 179.99 }, { lat: 0, lng: -179.99 })).toBeCloseTo(2223.9, 0);
    expect(distanceMeters({ lat: 90, lng: 0 }, { lat: 90, lng: 180 })).toBeLessThan(0.01);
    expect(distanceMeters(origin, { lat: 0, lng: 180 })).toBeCloseTo(Math.PI * 6_371_000, 0);
  });
});
