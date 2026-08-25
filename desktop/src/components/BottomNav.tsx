import { motion } from "framer-motion";
import { Activity, Info, Power, Settings as Cog } from "lucide-react";
import { springSoft } from "../lib/motion";
import type { Page } from "../lib/types";

const ITEMS: Array<{ id: Page; label: string; Icon: typeof Power }> = [
  { id: "connect", label: "Connect", Icon: Power },
  { id: "settings", label: "Settings", Icon: Cog },
  { id: "stats", label: "Stats", Icon: Activity },
  { id: "about", label: "About", Icon: Info },
];

/**
 * Bottom navigation with a single shared highlight pill that slides between
 * items via a layout animation, rather than four separately fading backgrounds.
 */
export function BottomNav({ page, onChange }: { page: Page; onChange: (p: Page) => void }) {
  return (
    <nav className="shrink-0 border-t border-hairline bg-black/25 px-2 py-1.5 backdrop-blur-xl">
      <div className="flex items-stretch">
        {ITEMS.map(({ id, label, Icon }) => {
          const active = page === id;
          return (
            <button
              key={id}
              type="button"
              onClick={() => onChange(id)}
              className="relative flex flex-1 flex-col items-center gap-0.5 rounded-lg py-1.5 outline-none"
            >
              {active && (
                <motion.span
                  layoutId="nav-pill"
                  className="absolute inset-0 rounded-lg bg-white/[0.07]"
                  transition={springSoft}
                />
              )}
              <Icon
                size={16}
                strokeWidth={2.2}
                className={`relative transition-colors ${active ? "text-ink" : "text-ink-faint"}`}
              />
              <span
                className={`relative text-[10px] font-medium transition-colors ${
                  active ? "text-ink" : "text-ink-faint"
                }`}
              >
                {label}
              </span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}
