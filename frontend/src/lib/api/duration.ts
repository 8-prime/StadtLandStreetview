// Search duration limits shared with the server (seconds).
export const minDuration = 60;
export const maxDuration = 24 * 60 * 60;

export const durationPresets: { seconds: number; label: string }[] = [
  { seconds: 60, label: "1 minute · test drive" },
  { seconds: 300, label: "5 minutes · quick snoop" },
  { seconds: 600, label: "10 minutes · proper investigation" },
  { seconds: 900, label: "15 minutes · deep dive" },
  { seconds: 1200, label: "20 minutes · leave no street unturned" },
];

export const clampDuration = (seconds: number) =>
  Math.min(
    maxDuration,
    Math.max(
      minDuration,
      Math.round(Number.isFinite(seconds) ? seconds : minDuration),
    ),
  );

/** A readable duration such as "10 minutes", "2 hours" or "1 h 30 min". */
export function formatDuration(seconds: number) {
  const minutes = Math.max(1, Math.round(seconds / 60));
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (!hours) return `${minutes} ${minutes === 1 ? "minute" : "minutes"}`;
  if (!rest) return `${hours} ${hours === 1 ? "hour" : "hours"}`;
  return `${hours} h ${rest} min`;
}

/** A countdown clock: mm:ss, or h:mm:ss once an hour or more remains. */
export function formatClock(seconds: number) {
  const s = Math.max(0, Math.floor(seconds));
  const pad = (value: number) => value.toString().padStart(2, "0");
  const hours = Math.floor(s / 3600);
  const clock = `${pad(Math.floor((s % 3600) / 60))}:${pad(s % 60)}`;
  return hours ? `${hours}:${clock}` : clock;
}
