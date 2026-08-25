import { motion } from "framer-motion";
import { Check, ShieldAlert } from "lucide-react";
import { Accordion, Field, TextField } from "../components/Field";
import { Segmented } from "../components/Segmented";
import { StepSlider } from "../components/StepSlider";
import { Toggle } from "../components/Toggle";
import { springSoft, staggerChild, staggerParent } from "../lib/motion";
import { useStore } from "../lib/store";
import {
  isBusy,
  OBFUSCATION_HINT,
  OBFUSCATION_LABEL,
  SCAN_HINT,
  SCAN_LABEL,
  SCAN_ORDER,
  type IpVersion,
  type Obfuscation,
  type Protocol,
} from "../lib/types";

export function SettingsPage() {
  const s = useStore((st) => st.settings);
  const patch = useStore((st) => st.patchSettings);
  const phase = useStore((st) => st.phase);
  const elevated = useStore((st) => st.elevated);

  // Changing transport-level settings mid-tunnel would desync the backend.
  const locked = phase === "connected" || isBusy(phase);

  return (
    <motion.div
      variants={staggerParent}
      initial="initial"
      animate="animate"
      className="flex h-full flex-col gap-3.5 overflow-y-auto px-4 pb-4"
    >
      {locked && (
        <motion.div
          variants={staggerChild}
          className="flex items-center gap-2 rounded-lg border border-warn/25 bg-warn/[0.08] px-2.5 py-2"
        >
          <ShieldAlert size={13} className="shrink-0 text-warn" />
          <span className="text-[10.5px] text-ink-dim">
            Disconnect to change connection settings.
          </span>
        </motion.div>
      )}

      <motion.div variants={staggerChild}>
        <Field label="Connection Mode">
          <div className="flex flex-col gap-1.5">
            <ModeOption
              active={s.mode === "proxy"}
              disabled={locked}
              onSelect={() => patch({ mode: "proxy" })}
              title="System Proxy"
              body="Fast, no admin rights. Only reaches apps that honour the Windows proxy — browsers, curl, most dev tools."
            />
            <ModeOption
              active={s.mode === "tun"}
              disabled={locked}
              onSelect={() => patch({ mode: "tun" })}
              title="Full Tunnel (TUN)"
              body="Routes every packet on the machine — games, Telegram Desktop, torrents, everything. Requires administrator."
              warn={!elevated ? "Not running as administrator" : undefined}
            />
          </div>
        </Field>
      </motion.div>

      <motion.div variants={staggerChild}>
        <Field label="Protocol">
          <Segmented<Protocol>
            id="protocol"
            value={s.protocol}
            onChange={(protocol) => !locked && patch({ protocol })}
            options={[
              { id: "masque", label: "MASQUE" },
              { id: "wireguard", label: "WireGuard" },
              { id: "gool", label: "gool" },
            ]}
          />
          <span className="px-1 text-[10px] leading-snug text-ink-faint">
            {s.protocol === "masque"
              ? "HTTP/3 over QUIC — looks like ordinary HTTPS. Best against DPI."
              : s.protocol === "wireguard"
                ? "Classic WireGuard — fastest, but easier to fingerprint."
                : "WireGuard nested inside WireGuard — slowest, hardest to block."}
          </span>
        </Field>
      </motion.div>

      <motion.div variants={staggerChild}>
        <Field label="Scan Mode" hint={SCAN_HINT[s.scan]}>
          <StepSlider
            value={s.scan}
            steps={SCAN_ORDER}
            labels={SCAN_LABEL}
            onChange={(scan) => !locked && patch({ scan })}
          />
        </Field>
      </motion.div>

      <motion.div variants={staggerChild}>
        <Field label="Obfuscation" hint={OBFUSCATION_HINT[s.obfuscation]}>
          <Segmented<Obfuscation>
            id="obfuscation"
            value={s.obfuscation}
            onChange={(obfuscation) => !locked && patch({ obfuscation })}
            options={(["off", "light", "balanced", "gfw"] as Obfuscation[]).map((id) => ({
              id,
              label: OBFUSCATION_LABEL[id],
            }))}
          />
        </Field>
      </motion.div>

      <motion.div variants={staggerChild} className="flex flex-col gap-1">
        <Toggle
          checked={s.killSwitch}
          onChange={(killSwitch) => patch({ killSwitch })}
          label="Kill Switch"
          hint="Drop the default route if the tunnel dies, so nothing leaks. Full Tunnel only."
          disabled={s.mode !== "tun"}
        />
        <Toggle
          checked={s.quickReconnect}
          onChange={(quickReconnect) => patch({ quickReconnect })}
          label="Quick reconnect"
          hint="Reuse the last working gateway instead of rescanning."
        />
        <Toggle
          checked={s.autoConnect}
          onChange={(autoConnect) => patch({ autoConnect })}
          label="Connect on launch"
        />
        <Toggle
          checked={s.startWithWindows}
          onChange={(startWithWindows) => patch({ startWithWindows })}
          label="Start with Windows"
          hint="Launch minimised to the tray at sign-in."
        />
      </motion.div>

      <motion.div variants={staggerChild}>
        <Accordion title="Advanced">
          <Field label="SOCKS5 bind address">
            <TextField
              value={s.bind}
              onChange={(bind) => patch({ bind })}
              placeholder="127.0.0.1:1819"
            />
          </Field>

          <Field label="IP version">
            <Segmented<IpVersion>
              id="ipver"
              value={s.ip}
              onChange={(ip) => !locked && patch({ ip })}
              options={[
                { id: "v4", label: "IPv4" },
                { id: "v6", label: "IPv6" },
                { id: "dual", label: "Dual" },
              ]}
            />
          </Field>

          <Field label="DNS inside the tunnel">
            <TextField value={s.dns} onChange={(dns) => patch({ dns })} placeholder="1.1.1.1, 1.0.0.1" />
          </Field>

          <Field label="Bypass the tunnel" hint="comma separated">
            <TextField
              value={s.routeDirect}
              onChange={(routeDirect) => patch({ routeDirect })}
              placeholder="private, *.ir, bank.example.com"
              mono={false}
            />
          </Field>

          <Field label="Block entirely" hint="comma separated">
            <TextField
              value={s.routeBlock}
              onChange={(routeBlock) => patch({ routeBlock })}
              placeholder="keyword:doubleclick, port:25"
              mono={false}
            />
          </Field>
        </Accordion>
      </motion.div>

      <motion.p variants={staggerChild} className="pt-1 text-center text-[10px] text-ink-faint">
        Made by Amirreza
      </motion.p>
    </motion.div>
  );
}

function ModeOption({
  active,
  disabled,
  onSelect,
  title,
  body,
  warn,
}: {
  active: boolean;
  disabled?: boolean;
  onSelect: () => void;
  title: string;
  body: string;
  warn?: string;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onSelect}
      className={`relative overflow-hidden rounded-card border px-3 py-2.5 text-left outline-none transition-colors ${
        active ? "border-idle-from/45 bg-idle-from/[0.09]" : "border-hairline bg-white/[0.02]"
      } ${disabled ? "opacity-45" : "hover:border-white/15"}`}
    >
      <div className="flex items-center gap-2">
        <span
          className={`flex h-4 w-4 shrink-0 items-center justify-center rounded-full border transition-colors ${
            active ? "grad-idle border-transparent" : "border-white/20"
          }`}
        >
          {active && (
            <motion.span initial={{ scale: 0 }} animate={{ scale: 1 }} transition={springSoft}>
              <Check size={10} strokeWidth={3.4} className="text-white" />
            </motion.span>
          )}
        </span>
        <span className="text-[12.5px] font-medium text-ink">{title}</span>
        {warn && (
          <span className="ml-auto rounded bg-warn/15 px-1.5 py-0.5 text-[9px] font-medium text-warn">
            {warn}
          </span>
        )}
      </div>
      <p className="mt-1 pl-6 text-[10.5px] leading-relaxed text-ink-faint">{body}</p>
    </button>
  );
}
