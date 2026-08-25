/** Formatting helpers shared by the speed cards, stats page and tray tooltip. */

const RATE_UNITS = ["B/s", "KB/s", "MB/s", "GB/s"];
const SIZE_UNITS = ["B", "KB", "MB", "GB", "TB"];

function scale(value: number, units: string[]): { value: string; unit: string } {
  let v = Math.max(0, value);
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  const digits = v >= 100 ? 0 : v >= 10 ? 1 : 2;
  return { value: v.toFixed(digits), unit: units[i] };
}

export function formatRate(bytesPerSecond: number) {
  return scale(bytesPerSecond, RATE_UNITS);
}

export function formatSize(bytes: number) {
  return scale(bytes, SIZE_UNITS);
}

/** hh:mm:ss, collapsing to mm:ss under an hour. */
export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${pad(h)}:${pad(m)}:${pad(sec)}` : `${pad(m)}:${pad(sec)}`;
}
