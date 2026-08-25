import { motion } from "framer-motion";
import { springSoft } from "../lib/motion";

/**
 * Segmented control with one sliding pill (layoutId), used for protocol and
 * IP-version pickers. Generic over the option id so callers stay type-safe.
 */
export function Segmented<T extends string>({
  value,
  options,
  onChange,
  id,
}: {
  value: T;
  options: Array<{ id: T; label: string }>;
  onChange: (v: T) => void;
  /** Unique per-instance so multiple segmented controls don't share one pill. */
  id: string;
}) {
  return (
    <div className="flex gap-1 rounded-xl border border-hairline bg-black/25 p-1">
      {options.map((opt) => {
        const active = opt.id === value;
        return (
          <button
            key={opt.id}
            type="button"
            onClick={() => onChange(opt.id)}
            className="relative flex-1 rounded-lg px-2 py-1.5 outline-none"
          >
            {active && (
              <motion.span
                layoutId={`seg-${id}`}
                className="grad-idle absolute inset-0 rounded-lg opacity-90"
                transition={springSoft}
              />
            )}
            <span
              className={`relative text-[11.5px] font-medium transition-colors ${
                active ? "text-white" : "text-ink-faint hover:text-ink-dim"
              }`}
            >
              {opt.label}
            </span>
          </button>
        );
      })}
    </div>
  );
}
