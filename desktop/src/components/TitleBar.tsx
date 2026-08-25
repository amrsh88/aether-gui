import { Minus, X } from "lucide-react";
import { api } from "../lib/api";

/**
 * Custom title bar. The window is frameless (`decorations: false`), so the whole
 * strip is a drag region except for the two buttons.
 *
 * There is no maximize button: the layout is designed around a fixed 420px column
 * and a maximized window would just stretch empty space.
 */
export function TitleBar() {
  return (
    <div className="drag-region flex h-10 shrink-0 items-center justify-between pl-4 pr-1 select-none">
      <div className="flex items-center gap-2">
        <span className="grad-live block h-2 w-2 rounded-full" />
        <span className="text-[13px] font-medium tracking-tight text-ink">Aether GUI</span>
        <span className="text-[11px] text-ink-faint">by NetRepublic</span>
      </div>

      <div className="no-drag flex items-center">
        <WinButton onClick={() => void api.minimize()} label="Minimize">
          <Minus size={13} strokeWidth={2.4} />
        </WinButton>
        <WinButton onClick={() => void api.hide()} label="Close to tray" danger>
          <X size={14} strokeWidth={2.4} />
        </WinButton>
      </div>
    </div>
  );
}

function WinButton({
  children,
  onClick,
  label,
  danger,
}: {
  children: React.ReactNode;
  onClick: () => void;
  label: string;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={`flex h-8 w-11 items-center justify-center rounded-md text-ink-faint transition-colors ${
        danger ? "hover:bg-danger hover:text-white" : "hover:bg-white/[0.07] hover:text-ink"
      }`}
    >
      {children}
    </button>
  );
}
