import { describe, expect, it } from "vitest";
import {
  clampDuration,
  formatClock,
  formatDuration,
  maxDuration,
} from "./duration";

describe("durations", () => {
  it("formats readable durations", () => {
    expect(formatDuration(60)).toBe("1 minute");
    expect(formatDuration(600)).toBe("10 minutes");
    expect(formatDuration(3600)).toBe("1 hour");
    expect(formatDuration(5400)).toBe("1 h 30 min");
    expect(formatDuration(maxDuration)).toBe("24 hours");
  });
  it("formats countdown clocks with hours when needed", () => {
    expect(formatClock(59)).toBe("00:59");
    expect(formatClock(1200)).toBe("20:00");
    expect(formatClock(3661)).toBe("1:01:01");
    expect(formatClock(maxDuration)).toBe("24:00:00");
  });
  it("clamps to the server limits", () => {
    expect(clampDuration(0)).toBe(60);
    expect(clampDuration(NaN)).toBe(60);
    expect(clampDuration(maxDuration + 60)).toBe(maxDuration);
  });
});
