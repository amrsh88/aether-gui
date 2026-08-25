import { motion } from "framer-motion";
import { Globe, Shield, ShieldOff, Zap } from "lucide-react";
import { staggerChild } from "../lib/motion";
import { PROTOCOL_LABEL, type PeerInfo, type TunnelMode } from "../lib/types";

/** Gateway + tunnel-mode summary shown once a tunnel is up. */
export function ServerChip({ peer, mode }: { peer: PeerInfo; mode: TunnelMode }) {
  const full = mode === "tun";

  return (
    <motion.div variants={staggerChild} className="flex w-full flex-col gap-2">
      <div className="glass glass-hover flex items-center gap-2.5 px-3 py-2.5">
        <Globe size={14} className="shrink-0 text-ink-dim" />
        <span className="tnum truncate text-[12px] text-ink">{peer.address}</span>
        <span className="ml-auto flex shrink-0 items-center gap-1">
          <Zap size={11} className="text-live-to" />
          <span className="tnum text-[12px] text-ink-dim">{peer.rttMs} ms</span>
        </span>
        <span className="shrink-0 rounded-md bg-white/[0.06] px-1.5 py-0.5 text-[10px] font-medium text-ink-dim">
          {PROTOCOL_LABEL[peer.protocol]}
        </span>
      </div>

      <div className="glass flex items-center gap-2.5 px-3 py-2">
        {full ? (
          <Shield size={13} className="shrink-0 text-live-to" />
        ) : (
          <ShieldOff size={13} className="shrink-0 text-warn" />
        )}
        <span className="text-[11px] text-ink-dim">
          {full ? "Full Tunnel — all system traffic" : "System Proxy — proxy-aware apps only"}
        </span>
      </div>
    </motion.div>
  );
}
