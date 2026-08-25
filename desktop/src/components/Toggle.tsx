import { motion } from "framer-motion";
import { springPress } from "../lib/motion";

/** iOS-style switch. Knob position is a spring so it overshoots very slightly. */
export function Toggle({
  checked,
  onChange,
  label,
  hint,
  disabled,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  hint?: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`flex w-full items-center gap-3 rounded-lg px-1 py-2 text-left outline-none transition-opacity ${
        disabled ? "opacity-40" : "hover:bg-white/[0.02]"
      }`}
    >
      <span className="flex-1">
        <span className="block text-[12.5px] text-ink">{label}</span>
        {hint && <span className="mt-0.5 block text-[10.5px] leading-snug text-ink-faint">{hint}</span>}
      </span>

      <span
        className={`relative flex h-[22px] w-[38px] shrink-0 items-center rounded-full px-[3px] transition-colors ${
          checked ? "grad-live" : "bg-white/[0.09]"
        }`}
      >
        <motion.span
          className="block h-4 w-4 rounded-full bg-white shadow-sm"
          animate={{ x: checked ? 16 : 0 }}
          transition={springPress}
        />
      </span>
    </button>
  );
}
