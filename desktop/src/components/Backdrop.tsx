import { motion } from "framer-motion";

/**
 * Two slow-drifting blurred blobs behind everything else. This is what stops the
 * window from looking like a flat rectangle — the light behind the glass moves.
 */
export function Backdrop({ live }: { live: boolean }) {
  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      <motion.div
        className="absolute h-[420px] w-[420px] rounded-full"
        style={{
          filter: "blur(90px)",
          background: live
            ? "radial-gradient(circle, rgba(6,182,212,0.34) 0%, transparent 68%)"
            : "radial-gradient(circle, rgba(99,102,241,0.30) 0%, transparent 68%)",
        }}
        animate={{
          x: ["-22%", "18%", "-12%", "-22%"],
          y: ["-18%", "6%", "22%", "-18%"],
        }}
        transition={{ duration: 26, repeat: Infinity, ease: "easeInOut" }}
      />
      <motion.div
        className="absolute right-0 bottom-0 h-[380px] w-[380px] rounded-full"
        style={{
          filter: "blur(90px)",
          background: live
            ? "radial-gradient(circle, rgba(16,185,129,0.26) 0%, transparent 68%)"
            : "radial-gradient(circle, rgba(139,92,246,0.24) 0%, transparent 68%)",
        }}
        animate={{
          x: ["16%", "-14%", "8%", "16%"],
          y: ["14%", "-8%", "-20%", "14%"],
        }}
        transition={{ duration: 32, repeat: Infinity, ease: "easeInOut" }}
      />
      {/* Faint vignette so the blobs never wash out the text. */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_35%,rgba(10,10,15,0.65)_100%)]" />
    </div>
  );
}
