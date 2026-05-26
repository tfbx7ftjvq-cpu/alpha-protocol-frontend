export const TOTAL_NETWORK_POINTS = 1_847_320;

export function getDaysMultiplier(days: number): number {
  if (days >= 180) return 8;
  if (days >= 90)  return 4;
  if (days >= 30)  return 2;
  return 1;
}

export const pad = (n: number): string => String(n).padStart(2, '0');
