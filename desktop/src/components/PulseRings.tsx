import { motion } from "framer-motion";

/**
 * Concentric rings that breathe outward from the power button.
 *
 * Idle: nothing (the button is still).
 * Busy: fast, tight rings — reads as "working".
 * Connected: slow, wide rings — reads as "alive and healthy".
 */
export function PulseRings({ state }: { state: "off" | "busy" | "live" }) {
  if (state === "off") return null;

  const busy = state === "busy";
  const color = busy ? "rgba(139,92,246,0.55)" : "rgba(16,185,129,0.5)";
  const duration = busy ? 1.6 : 3.2;
  const maxScale = busy ? 1.55 : 1.95;

  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
      {[0, 1, 2].map((i) => (
        <motion.span
          key={i}
          className="absolute rounded-full border"
          style={{
            width: 168,
            height: 168,
            borderColor: color,
          }}
          initial={{ scale: 1, opacity: 0 }}
          animate={{ scale: [1, maxScale], opacity: [0.6, 0] }}
          transition={{
            duration,
            repeat: Infinity,
            ease: "easeOut",
            delay: (duration / 3) * i,
          }}
        />
      ))}
    </div>
  );
}
