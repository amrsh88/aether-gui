/**
 * The single source of truth for everything that crosses the Rust <-> UI border.
 * Field names here must match the serde representation in src-tauri/src/model.rs.
 */

export type Protocol = "masque" | "wireguard" | "gool";

export type ScanMode = "turbo" | "balanced" | "thorough" | "stealth" | "ironclad";

export type Obfuscation = "off" | "light" | "balanced" | "gfw";

export type IpVersion = "v4" | "v6" | "dual";

export type TunnelMode = "proxy" | "tun";

/** Which screen the bottom nav is showing. */
export type Page = "connect" | "settings" | "stats" | "about";

/** Coarse lifecycle state driven entirely by the backend. */
export type Phase =
  | "idle"
  | "starting"
  | "scanning"
  | "verifying"
  | "routing"
  | "connected"
  | "stopping"
  | "error";

export interface Settings {
  mode: TunnelMode;
  protocol: Protocol;
  scan: ScanMode;
  obfuscation: Obfuscation;
  bind: string;
  httpProxy: string | null;
  ip: IpVersion;
  dns: string;
  killSwitch: boolean;
  autoConnect: boolean;
  startWithWindows: boolean;
  quickReconnect: boolean;
  routeDirect: string;
  routeBlock: string;
}

export const defaultSettings: Settings = {
  mode: "proxy",
  protocol: "masque",
  scan: "balanced",
  obfuscation: "balanced",
  bind: "127.0.0.1:1819",
  httpProxy: null,
  ip: "v4",
  dns: "1.1.1.1, 1.0.0.1",
  killSwitch: true,
  autoConnect: false,
  startWithWindows: false,
  quickReconnect: true,
  routeDirect: "",
  routeBlock: "",
};

/** The gateway we ended up connected through. */
export interface PeerInfo {
  address: string;
  rttMs: number;
  protocol: Protocol;
}

/** One sample of the throughput meter, emitted roughly once per second. */
export interface Sample {
  t: number;
  down: number;
  up: number;
}

export interface Totals {
  down: number;
  up: number;
}

/** A backend progress event. `detail` is already human-readable English. */
export interface ProgressEvent {
  phase: Phase;
  detail: string;
  peer?: PeerInfo | null;
}

export interface StatsEvent {
  downBps: number;
  upBps: number;
  totalDown: number;
  totalUp: number;
}

export interface LogLine {
  id: number;
  level: "info" | "warn" | "error" | "debug";
  text: string;
  at: number;
}

export interface ScannedPeer {
  address: string;
  rttMs: number;
  ok: boolean;
}

export const PROTOCOL_LABEL: Record<Protocol, string> = {
  masque: "MASQUE",
  wireguard: "WireGuard",
  gool: "gool",
};

export const SCAN_ORDER: ScanMode[] = ["turbo", "balanced", "thorough", "stealth", "ironclad"];

export const SCAN_LABEL: Record<ScanMode, string> = {
  turbo: "Turbo",
  balanced: "Balanced",
  thorough: "Thorough",
  stealth: "Stealth",
  ironclad: "Ironclad",
};

export const SCAN_HINT: Record<ScanMode, string> = {
  turbo: "Fewest probes, connects fastest",
  balanced: "Good default for most networks",
  thorough: "More candidates, slower but steadier",
  stealth: "Quiet probing, avoids burst detection",
  ironclad: "Real tunnel + HTTP check per candidate",
};

export const OBFUSCATION_LABEL: Record<Obfuscation, string> = {
  off: "Off",
  light: "Light",
  balanced: "Balanced",
  gfw: "GFW",
};

export const OBFUSCATION_HINT: Record<Obfuscation, string> = {
  off: "No padding — fastest, most fingerprintable",
  light: "Light padding for ordinary firewalls",
  balanced: "Recommended for Iranian ISPs",
  gfw: "Heaviest obfuscation, costs throughput",
};

/** Phase -> label shown under the power button. */
export const PHASE_LABEL: Record<Phase, string> = {
  idle: "Disconnected",
  starting: "Starting",
  scanning: "Scanning for gateways",
  verifying: "Verifying tunnel",
  routing: "Applying routes",
  connected: "Connected",
  stopping: "Disconnecting",
  error: "Failed",
};

/** Phases where the engine is doing work and the button should show motion. */
export const BUSY_PHASES: Phase[] = ["starting", "scanning", "verifying", "routing", "stopping"];

export function isBusy(phase: Phase): boolean {
  return BUSY_PHASES.includes(phase);
}
