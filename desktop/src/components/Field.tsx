import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { collapse } from "../lib/motion";

/** Section header with a label and optional trailing hint. */
export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-baseline justify-between">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-ink-faint">
          {label}
        </span>
        {hint && <span className="text-[10px] text-ink-faint">{hint}</span>}
      </div>
      {children}
    </div>
  );
}

/** Single-line text input styled to match the glass panels. */
export function TextField({
  value,
  onChange,
  placeholder,
  mono = true,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  mono?: boolean;
}) {
  return (
    <input
      type="text"
      value={value}
      placeholder={placeholder}
      spellCheck={false}
      onChange={(e) => onChange(e.target.value)}
      className={`w-full rounded-lg border border-hairline bg-black/25 px-2.5 py-2 text-[12px] text-ink outline-none transition-colors placeholder:text-ink-faint focus:border-idle-from/60 ${
        mono ? "tnum" : ""
      }`}
      style={{ WebkitUserSelect: "text", userSelect: "text" }}
    />
  );
}

/** Collapsible section used for the advanced settings block. */
export function Accordion({
  title,
  children,
  defaultOpen = false,
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="glass overflow-hidden">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="flex w-full items-center justify-between px-3 py-2.5 outline-none"
      >
        <span className="text-[10px] font-semibold uppercase tracking-wider text-ink-faint">
          {title}
        </span>
        <motion.span animate={{ rotate: open ? 180 : 0 }} transition={{ duration: 0.22 }}>
          <ChevronDown size={14} className="text-ink-faint" />
        </motion.span>
      </button>

      <AnimatePresence initial={false}>
        {open && (
          <motion.div variants={collapse} initial="initial" animate="animate" exit="exit">
            <div className="flex flex-col gap-3 border-t border-hairline px-3 py-3">{children}</div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
