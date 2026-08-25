import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ProgressEvent,
  ScannedPeer,
  Settings,
  StatsEvent,
} from "./types";

/**
 * Thin typed wrapper over the Tauri command surface.
 *
 * Nothing else in the UI is allowed to call `invoke` directly — keeping it here
 * means the whole IPC contract is visible in one file, and the dev-mode mock
 * below can stand in for the backend when running `npm run dev` in a browser.
 */

/** True when running in a plain browser (vite dev, no Tauri runtime attached). */
export const isMock = !("__TAURI_INTERNALS__" in window);

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isMock) return mockCall<T>(cmd, args);
  return invoke<T>(cmd, args);
}

export const api = {
  connect: (settings: Settings) => call<void>("connect", { settings }),
  disconnect: () => call<void>("disconnect"),
  loadSettings: () => call<Settings | null>("load_settings"),
  saveSettings: (settings: Settings) => call<void>("save_settings", { settings }),
  scannedPeers: () => call<ScannedPeer[]>("scanned_peers"),
  openUrl: (url: string) => call<void>("open_url", { url }),
  isElevated: () => call<boolean>("is_elevated"),
  version: () => call<string>("core_version"),
  minimize: () => call<void>("win_minimize"),
  hide: () => call<void>("win_hide"),
};

export function onProgress(fn: (e: ProgressEvent) => void): Promise<UnlistenFn> {
  if (isMock) return mockListen("aether://progress", fn);
  return listen<ProgressEvent>("aether://progress", (e) => fn(e.payload));
}

export function onStats(fn: (e: StatsEvent) => void): Promise<UnlistenFn> {
  if (isMock) return mockListen("aether://stats", fn);
  return listen<StatsEvent>("aether://stats", (e) => fn(e.payload));
}

export function onLog(fn: (line: string) => void): Promise<UnlistenFn> {
  if (isMock) return mockListen("aether://log", fn);
  return listen<string>("aether://log", (e) => fn(e.payload));
}

/* ------------------------------------------------------------------ *
 * Browser mock — lets the UI be developed and reviewed without Windows
 * ------------------------------------------------------------------ */

type Handler = (payload: unknown) => void;
const mockHandlers = new Map<string, Set<Handler>>();
let mockTimer: number | null = null;

function emit(channel: string, payload: unknown) {
  mockHandlers.get(channel)?.forEach((h) => h(payload));
}

function mockListen<T>(channel: string, fn: (p: T) => void): Promise<UnlistenFn> {
  const set = mockHandlers.get(channel) ?? new Set<Handler>();
  set.add(fn as Handler);
  mockHandlers.set(channel, set);
  return Promise.resolve(() => {
    set.delete(fn as Handler);
  });
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function mockCall<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case "connect": {
      const steps: Array<[string, string, number]> = [
        ["starting", "Loading identity", 500],
        ["scanning", "Probing 128 candidates", 1800],
        ["verifying", "Validating data plane", 900],
        ["routing", "Applying system routes", 600],
      ];
      for (const [phase, detail, wait] of steps) {
        emit("aether://progress", { phase, detail });
        emit("aether://log", `[+] ${detail}`);
        await sleep(wait);
      }
      emit("aether://progress", {
        phase: "connected",
        detail: "Tunnel is up",
        peer: { address: "162.159.192.7:443", rttMs: 42, protocol: "masque" },
      });
      startMockStats();
      return undefined as T;
    }
    case "disconnect": {
      emit("aether://progress", { phase: "stopping", detail: "Tearing down routes" });
      stopMockStats();
      await sleep(500);
      emit("aether://progress", { phase: "idle", detail: "" });
      return undefined as T;
    }
    case "load_settings":
      return null as T;
    case "save_settings":
      return undefined as T;
    case "scanned_peers":
      return [
        { address: "162.159.192.7:443", rttMs: 42, ok: true },
        { address: "162.159.193.10:2408", rttMs: 88, ok: true },
        { address: "188.114.96.3:443", rttMs: 131, ok: false },
      ] as T;
    case "open_url":
      window.open(String(args?.url ?? ""), "_blank");
      return undefined as T;
    case "is_elevated":
      return false as T;
    case "core_version":
      return "1.7.0 (mock)" as T;
    default:
      return undefined as T;
  }
}

let mockTotalDown = 0;
let mockTotalUp = 0;

function startMockStats() {
  stopMockStats();
  mockTimer = window.setInterval(() => {
    const downBps = 6_000_000 + Math.random() * 9_000_000;
    const upBps = 800_000 + Math.random() * 2_400_000;
    mockTotalDown += downBps;
    mockTotalUp += upBps;
    emit("aether://stats", {
      downBps,
      upBps,
      totalDown: mockTotalDown,
      totalUp: mockTotalUp,
    });
  }, 1000);
}

function stopMockStats() {
  if (mockTimer !== null) {
    window.clearInterval(mockTimer);
    mockTimer = null;
  }
}
