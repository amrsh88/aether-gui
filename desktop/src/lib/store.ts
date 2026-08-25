import { create } from "zustand";
import { api, onLog, onProgress, onStats } from "./api";
import {
  defaultSettings,
  type LogLine,
  type Page,
  type PeerInfo,
  type Phase,
  type Sample,
  type Settings,
  type Totals,
} from "./types";

export type { Page };

const MAX_SAMPLES = 60;
const MAX_LOGS = 400;

interface State {
  page: Page;
  phase: Phase;
  detail: string;
  peer: PeerInfo | null;
  settings: Settings;
  samples: Sample[];
  totals: Totals;
  downBps: number;
  upBps: number;
  connectedAt: number | null;
  elapsed: number;
  logs: LogLine[];
  toast: { text: string; kind: "error" | "info" } | null;
  elevated: boolean;
  coreVersion: string;

  setPage: (p: Page) => void;
  patchSettings: (patch: Partial<Settings>) => void;
  toggle: () => Promise<void>;
  dismissToast: () => void;
  init: () => Promise<void>;
  tick: () => void;
}

let logSeq = 0;

export const useStore = create<State>((set, get) => ({
  page: "connect",
  phase: "idle",
  detail: "",
  peer: null,
  settings: defaultSettings,
  samples: [],
  totals: { down: 0, up: 0 },
  downBps: 0,
  upBps: 0,
  connectedAt: null,
  elapsed: 0,
  logs: [],
  toast: null,
  elevated: false,
  coreVersion: "",

  setPage: (page) => set({ page }),

  patchSettings: (patch) => {
    const settings = { ...get().settings, ...patch };
    set({ settings });
    void api.saveSettings(settings);
  },

  dismissToast: () => set({ toast: null }),

  tick: () => {
    const at = get().connectedAt;
    if (at === null) return;
    set({ elapsed: Math.floor((Date.now() - at) / 1000) });
  },

  toggle: async () => {
    const { phase, settings } = get();

    // Busy phases are treated as "cancel", so a stuck scan is always escapable.
    if (phase === "connected" || phase === "scanning" || phase === "verifying" || phase === "starting" || phase === "routing") {
      await api.disconnect();
      return;
    }

    if (settings.mode === "tun" && !get().elevated) {
      set({
        toast: {
          kind: "error",
          text: "Full Tunnel needs administrator rights. Restart Aether GUI as administrator.",
        },
      });
      return;
    }

    set({ samples: [], totals: { down: 0, up: 0 }, elapsed: 0 });
    try {
      await api.connect(settings);
    } catch (e) {
      set({ phase: "error", detail: String(e), toast: { kind: "error", text: String(e) } });
    }
  },

  init: async () => {
    const [stored, elevated, coreVersion] = await Promise.all([
      api.loadSettings(),
      api.isElevated(),
      api.version(),
    ]);
    set({
      settings: stored ? { ...defaultSettings, ...stored } : defaultSettings,
      elevated,
      coreVersion,
    });

    await onProgress((e) => {
      const patch: Partial<State> = { phase: e.phase, detail: e.detail };
      if (e.peer !== undefined) patch.peer = e.peer;

      if (e.phase === "connected") {
        patch.connectedAt = Date.now();
      } else if (e.phase === "idle" || e.phase === "error") {
        patch.connectedAt = null;
        patch.elapsed = 0;
        patch.downBps = 0;
        patch.upBps = 0;
        patch.peer = null;
      }
      if (e.phase === "error" && e.detail) {
        patch.toast = { kind: "error", text: e.detail };
      }
      set(patch);
    });

    await onStats((s) => {
      const samples = [...get().samples, { t: Date.now(), down: s.downBps, up: s.upBps }];
      set({
        downBps: s.downBps,
        upBps: s.upBps,
        totals: { down: s.totalDown, up: s.totalUp },
        samples: samples.slice(-MAX_SAMPLES),
      });
    });

    await onLog((text) => {
      const level: LogLine["level"] = text.includes("[-]")
        ? "error"
        : text.includes("[!]")
          ? "warn"
          : "info";
      const logs = [...get().logs, { id: ++logSeq, level, text, at: Date.now() }];
      set({ logs: logs.slice(-MAX_LOGS) });
    });

    if (get().settings.autoConnect) {
      void get().toggle();
    }
  },
}));
