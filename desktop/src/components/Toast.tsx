import { AnimatePresence, motion } from "framer-motion";
import { AlertTriangle, Info, X } from "lucide-react";
import { toastRise } from "../lib/motion";

/** Single transient message docked above the bottom nav. */
export function Toast({
  toast,
  onDismiss,
}: {
  toast: { text: string; kind: "error" | "info" } | null;
  onDismiss: () => void;
}) {
  return (
    <AnimatePresence>
      {toast && (
        <motion.div
          variants={toastRise}
          initial="initial"
          animate="animate"
          exit="exit"
          className="absolute inset-x-4 bottom-3 z-20"
        >
          <div
            className="flex items-start gap-2.5 rounded-card border px-3 py-2.5 backdrop-blur-xl"
            style={{
              background: toast.kind === "error" ? "rgba(239,68,68,0.14)" : "rgba(255,255,255,0.06)",
              borderColor: toast.kind === "error" ? "rgba(239,68,68,0.32)" : "rgba(255,255,255,0.1)",
            }}
          >
            {toast.kind === "error" ? (
              <AlertTriangle size={14} className="mt-0.5 shrink-0 text-danger" />
            ) : (
              <Info size={14} className="mt-0.5 shrink-0 text-ink-dim" />
            )}
            <p className="flex-1 text-[11.5px] leading-relaxed text-ink">{toast.text}</p>
            <button
              type="button"
              onClick={onDismiss}
              aria-label="Dismiss"
              className="shrink-0 rounded p-0.5 text-ink-faint transition-colors hover:text-ink"
            >
              <X size={13} />
            </button>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
