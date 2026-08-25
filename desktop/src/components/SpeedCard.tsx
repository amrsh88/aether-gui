import { motion } from "framer-motion";
import { ArrowDown, ArrowUp } from "lucide-react";
import { formatRate } from "../lib/format";
import { staggerChild } from "../lib/motion";
import { Sparkline } from "./Sparkline";

/** One throughput card: rate, unit, and a sparkline of recent history. */
export function SpeedCard({
  direction,
  bps,
  history,
}: {
  direction: "down" | "up";
  bps: number;
  history: number[];
}) {
  const down = direction === "down";
  const { value, unit } = formatRate(bps);
  const color = down ? "#06B6D4" : "#8B5CF6";
  const Icon = down ? ArrowDown : ArrowUp;

  return (
    <motion.div variants={staggerChild} className="glass glass-hover flex-1 overflow-hidden p-3">
      <div className="mb-1.5 flex items-center gap-1.5">
        <Icon size={12} strokeWidth={2.6} style={{ color }} />
        <span className="text-[10px] font-medium uppercase tracking-wider text-ink-faint">
          {down ? "Download" : "Upload"}
        </span>
      </div>

      <div className="flex items-baseline gap-1">
        <span className="tnum text-[20px] font-semibold leading-none text-ink">{value}</span>
        <span className="text-[11px] text-ink-faint">{unit}</span>
      </div>

      <div className="mt-1.5">
        <Sparkline values={history} color={color} height={26} />
      </div>
    </motion.div>
  );
}
