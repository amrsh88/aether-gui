import { motion } from "framer-motion";
import { springSoft } from "../lib/motion";

/**
 * Discrete slider over N labelled stops (used for scan mode).
 *
 * Implemented as clickable dots plus a spring-driven knob instead of a native
 * <input type="range">, so the fill and knob can be styled and animated.
 */
export function StepSlider<T extends string>({
  value,
  steps,
  labels,
  onChange,
}: {
  value: T;
  steps: T[];
  labels: Record<T, string>;
  onChange: (v: T) => void;
}) {
  const index = Math.max(0, steps.indexOf(value));
  const last = steps.length - 1;
  const percent = last === 0 ? 0 : (index / last) * 100;

  return (
    <div className="px-1 pt-1">
      <div className="relative h-6">
        {/* Track */}
        <div className="absolute top-1/2 h-1 w-full -translate-y-1/2 rounded-full bg-white/[0.08]" />

        {/* Filled portion */}
        <motion.div
          className="grad-idle absolute top-1/2 h-1 -translate-y-1/2 rounded-full"
          animate={{ width: `${percent}%` }}
          transition={springSoft}
        />

        {/* Stops */}
        <div className="absolute inset-0 flex items-center justify-between">
          {steps.map((step, i) => (
            <button
              key={step}
              type="button"
              aria-label={labels[step]}
              onClick={() => onChange(step)}
              className="relative z-10 flex h-6 w-6 items-center justify-center outline-none"
            >
              <span
                className={`block rounded-full transition-all ${
                  i <= index ? "h-2 w-2 bg-white" : "h-1.5 w-1.5 bg-white/25"
                }`}
              />
            </button>
          ))}
        </div>

        {/* Knob */}
        <motion.div
          className="pointer-events-none absolute top-1/2 z-20 h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white bg-idle-to shadow-lg"
          animate={{ left: `${percent}%` }}
          transition={springSoft}
        />
      </div>

      <div className="mt-1 flex justify-between">
        {steps.map((step, i) => (
          <span
            key={step}
            className={`text-[9.5px] transition-colors ${
              i === index ? "font-medium text-ink" : "text-ink-faint"
            }`}
          >
            {labels[step]}
          </span>
        ))}
      </div>
    </div>
  );
}
