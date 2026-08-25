import { motion } from "framer-motion";
import { useEffect, useState } from "react";
import { Copy, Radio } from "lucide-react";
import { Sparkline } from "../components/Sparkline";
import { formatDuration, formatSize } from "../lib/format";
import { api } from "../lib/api";
import { staggerChild, staggerParent } from "../lib/motion";
import { useStore } from "../lib/store";
import type { ScannedPeer } from "../lib/types";

export function StatsPage() {
  const samples = useStore((s) => s.samples);
  const totals = useStore((s) => s.totals);
  const elapsed = useStore((s) => s.elapsed);
  const logs = useStore((s) => s.logs);
  const phase = useStore((s) => s.phase);

  const [peers, setPeers] = useState<ScannedPeer[]>([]);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    void api.scannedPeers().then(setPeers);
  }, [phase]);

  const down = formatSize(totals.down);
  const up = formatSize(totals.up);
  const peakDown = samples.length ? Math.max(...samples.map((s) => s.down)) : 0;
  const peak = formatSize(peakDown);

  async function copyLogs() {
    await navigator.clipboard.writeText(logs.map((l) => l.text).join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 1600);
  }

  return (
    <motion.div
      variants={staggerParent}
      initial="initial"
      animate="animate"
      className="flex h-full flex-col gap-3 overflow-y-auto px-4 pb-4"
    >
      <motion.div variants={staggerChild} className="grid grid-cols-3 gap-2">
        <Stat label="Downloaded" value={down.value} unit={down.unit} />
        <Stat label="Uploaded" value={up.value} unit={up.unit} />
        <Stat label="Uptime" value={formatDuration(elapsed)} unit="" />
      </motion.div>

      <motion.div variants={staggerChild} className="glass p-3">
        <div className="mb-2 flex items-baseline justify-between">
          <span className="text-[10px] font-semibold uppercase tracking-wider text-ink-faint">
            Throughput · last {samples.length}s
          </span>
          <span className="text-[10px] text-ink-faint">
            peak {peak.value} {peak.unit}/s
          </span>
        </div>
        <Sparkline values={samples.map((s) => s.down)} color="#06B6D4" height={64} />
        <div className="mt-1.5 flex gap-3">
          <Legend color="#06B6D4" label="Download" />
          <Legend color="#8B5CF6" label="Upload" />
        </div>
        <Sparkline values={samples.map((s) => s.up)} color="#8B5CF6" height={34} />
      </motion.div>

      <motion.div variants={staggerChild} className="glass overflow-hidden">
        <div className="flex items-center gap-1.5 px-3 py-2.5">
          <Radio size={11} className="text-ink-faint" />
          <span className="text-[10px] font-semibold uppercase tracking-wider text-ink-faint">
            Probed gateways
          </span>
        </div>
        <div className="border-t border-hairline">
          {peers.length === 0 ? (
            <p className="px-3 py-3 text-[11px] text-ink-faint">No scan has run yet.</p>
          ) : (
            peers.map((p) => (
              <div
                key={p.address}
                className="flex items-center gap-2 border-b border-hairline px-3 py-2 last:border-0"
              >
                <span
                  className="h-1.5 w-1.5 shrink-0 rounded-full"
                  style={{ background: p.ok ? "#10B981" : "#EF4444" }}
                />
                <span className="tnum flex-1 truncate text-[11px] text-ink-dim">{p.address}</span>
                <span className="tnum text-[11px] text-ink-faint">{p.rttMs} ms</span>
              </div>
            ))
          )}
        </div>
      </motion.div>

      <motion.div variants={staggerChild} className="glass overflow-hidden">
        <div className="flex items-center justify-between px-3 py-2.5">
          <span className="text-[10px] font-semibold uppercase tracking-wider text-ink-faint">
            Engine log
          </span>
          <button
            type="button"
            onClick={() => void copyLogs()}
            className="flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] text-ink-faint transition-colors hover:bg-white/[0.06] hover:text-ink"
          >
            <Copy size={10} />
            {copied ? "Copied" : "Copy"}
          </button>
        </div>
        <div className="max-h-[180px] overflow-y-auto border-t border-hairline bg-black/25 px-3 py-2">
          {logs.length === 0 ? (
            <p className="text-[10.5px] text-ink-faint">Nothing logged yet.</p>
          ) : (
            logs.map((l) => (
              <p
                key={l.id}
                className="tnum whitespace-pre-wrap break-all text-[10px] leading-relaxed"
                style={{
                  color:
                    l.level === "error" ? "#F87171" : l.level === "warn" ? "#FBBF24" : "#A1A1AA",
                }}
              >
                {l.text}
              </p>
            ))
          )}
        </div>
      </motion.div>
    </motion.div>
  );
}

function Stat({ label, value, unit }: { label: string; value: string; unit: string }) {
  return (
    <div className="glass glass-hover px-2.5 py-2">
      <p className="text-[9.5px] font-medium uppercase tracking-wider text-ink-faint">{label}</p>
      <p className="mt-0.5 flex items-baseline gap-0.5">
        <span className="tnum text-[15px] font-semibold text-ink">{value}</span>
        {unit && <span className="text-[10px] text-ink-faint">{unit}</span>}
      </p>
    </div>
  );
}

function Legend({ color, label }: { color: string; label: string }) {
  return (
    <span className="flex items-center gap-1">
      <span className="h-0.5 w-3 rounded" style={{ background: color }} />
      <span className="text-[9.5px] text-ink-faint">{label}</span>
    </span>
  );
}
