import { motion } from "framer-motion";
import { Power } from "lucide-react";
import { springPress } from "../lib/motion";
import { isBusy, type Phase } from "../lib/types";
import { PulseRings } from "./PulseRings";

/**
 * The centrepiece control.
 *
 * One button, four visual states, and a rotating conic ring that only spins
 * while the engine is actually doing work. Clicking during a busy phase cancels,
 * so the user is never trapped watching a scan they no longer want.
 */
export function PowerButton({
  phase,
  onClick,
}: {
  phase: Phase;
  onClick: () => void;
}) {
  const busy = isBusy(phase);
  const live = phase === "connected";
  const failed = phase === "error";

  const ringState = live ? "live" : busy ? "busy" : "off";
  const gradient = failed ? "grad-danger" : live ? "grad-live" : "grad-idle";
  const glow = failed
    ? "rgba(239,68,68,0.5)"
    : live
      ? "rgba(16,185,129,0.55)"
      : "rgba(99,102,241,0.5)";

  return (
    <div className="relative flex h-[220px] w-[220px] items-center justify-center">
      <PulseRings state={ringState} />

      {/* Sweeping conic ring — the "we are scanning" signal. */}
      {busy && (
        <motion.div
          className="absolute h-[190px] w-[190px] rounded-full"
          style={{
            background:
              "conic-gradient(from 0deg, transparent 0deg, rgba(139,92,246,0.9) 90deg, transparent 180deg)",
            maskImage: "radial-gradient(circle, transparent 62%, black 64%, black 68%, transparent 70%)",
            WebkitMaskImage:
              "radial-gradient(circle, transparent 62%, black 64%, black 68%, transparent 70%)",
          }}
          animate={{ rotate: 360 }}
          transition={{ duration: 1.4, repeat: Infinity, ease: "linear" }}
        />
      )}

      {/* Static rim so the button has an edge even when idle. */}
      <div
        className="absolute h-[188px] w-[188px] rounded-full border"
        style={{
          borderColor: live ? "rgba(16,185,129,0.22)" : "rgba(255,255,255,0.07)",
        }}
      />

      <motion.button
        type="button"
        onClick={onClick}
        aria-label={live ? "Disconnect" : busy ? "Cancel" : "Connect"}
        className={`${gradient} relative flex h-[160px] w-[160px] items-center justify-center rounded-full outline-none`}
        style={{ boxShadow: `0 0 56px -10px ${glow}, inset 0 1px 0 rgba(255,255,255,0.22)` }}
        whileHover={{ scale: 1.03 }}
        whileTap={{ scale: 0.94 }}
        animate={
          live
            ? {
                boxShadow: [
                  `0 0 46px -12px ${glow}`,
                  `0 0 78px -8px ${glow}`,
                  `0 0 46px -12px ${glow}`,
                ],
              }
            : {}
        }
        transition={
          live
            ? { ...springPress, boxShadow: { duration: 3, repeat: Infinity, ease: "easeInOut" } }
            : springPress
        }
      >
        {/* Inner disc keeps the icon readable against the gradient. */}
        <span className="absolute inset-[6px] rounded-full bg-black/25 backdrop-blur-[2px]" />
        <motion.span
          className="relative"
          animate={busy ? { opacity: [1, 0.45, 1] } : { opacity: 1 }}
          transition={busy ? { duration: 1.4, repeat: Infinity, ease: "easeInOut" } : { duration: 0.2 }}
        >
          <Power size={54} strokeWidth={2.1} className="text-white drop-shadow-lg" />
        </motion.span>
      </motion.button>
    </div>
  );
}
