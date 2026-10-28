export type SpotterMood = 'idle' | 'ready' | 'searching' | 'winner' | 'offline';

// A stable identity, not a new random draw on each render or client.
export function makeSpotter(seed: string) {
  let state = 2166136261;
  for (const character of seed) state = Math.imul(state ^ character.charCodeAt(0), 16777619);
  const pick = (count: number) => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) % count;
  };
  return {
    skin: ['#f6c9a3', '#d99a70', '#a96e4e', '#754d3c', '#edb68e'][pick(5)],
    hair: ['#24304a', '#6c4435', '#c36b3e', '#e6b958', '#eee2c5'][pick(5)],
    shirt: ['#ffcc3e', '#f88472', '#86c7b0', '#a9a0eb', '#71b9d5', '#e99dc2'][pick(6)],
    backdrop: ['#b6caf6', '#f6d588', '#f4b4a4', '#a9d6c3', '#c9bdeb', '#bddde8'][pick(6)],
    hat: pick(4), hairStyle: pick(3), glasses: pick(3), accessory: pick(3), smile: pick(3),
  };
}
