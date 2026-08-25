import { AnimatePresence, motion } from "framer-motion";
import { easeOut, swapText } from "../lib/motion";
import { formatDuration } from "../lib/format";
import { isBusy, PHASE_LABEL, type Phase } from "../lib/types";

/**
 * The status block under the button: headline, then either the live timer or the
 * backend's current detail line. Both swap with a blur so nothing ever pops.
 */
export function StatusText({
  phase,
  detail,
  elapsed,
}: {
  phase: Phase;
  detail: string;
  elapsed: number;
}) {
  const busy = isBusy(phase);
  const live = phase === "connected";
  const headline = PHASE_LABEL[phase];

  return (
    <div className="flex h-[62px] flex-col items-center justify-center gap-1">
      <AnimatePresence mode="wait">
        <motion.div
          key={headline}
          variants={swapText}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={easeOut}
          className={`text-[19px] font-semibold tracking-tight ${
            live ? "text-grad-live" : phase === "error" ? "text-danger" : "text-ink"
          } ${busy ? "shimmer" : ""}`}
        >
          {headline}
        </motion.div>
      </AnimatePresence>

      <AnimatePresence mode="wait">
        {live ? (
          <motion.div
            key="timer"
            variants={swapText}
            initial="initial"
            animate="animate"
            exit="exit"
            transition={easeOut}
            className="tnum text-[13px] text-ink-dim"
          >
            {formatDuration(elapsed)}
          </motion.div>
        ) : detail ? (
          <motion.div
            key={detail}
            variants={swapText}
            initial="initial"
            animate="animate"
            exit="exit"
            transition={easeOut}
            className="max-w-[300px] truncate text-center text-[12px] text-ink-faint"
          >
            {detail}
          </motion.div>
        ) : (
          <motion.div
            key="hint"
            variants={swapText}
            initial="initial"
            animate="animate"
            exit="exit"
            transition={easeOut}
            className="text-[12px] text-ink-faint"
          >
            Tap to connect
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
