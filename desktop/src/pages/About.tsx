import { motion } from "framer-motion";
import { ExternalLink, Github, Heart, Send } from "lucide-react";
import { api } from "../lib/api";
import { staggerChild, staggerParent } from "../lib/motion";
import { useStore } from "../lib/store";

const GITHUB_UPSTREAM = "https://github.com/CluvexStudio/Aether";
const TELEGRAM_CHANNEL = "https://t.me/net_republic";

export function AboutPage() {
  const coreVersion = useStore((s) => s.coreVersion);

  return (
    <motion.div
      variants={staggerParent}
      initial="initial"
      animate="animate"
      className="flex h-full flex-col gap-3 overflow-y-auto px-4 pb-4"
    >
      <motion.div variants={staggerChild} className="flex flex-col items-center pt-3 pb-1">
        <motion.div
          className="grad-live mb-3 flex h-14 w-14 items-center justify-center rounded-2xl"
          style={{ boxShadow: "0 0 34px -8px rgba(16,185,129,0.55)" }}
          animate={{ y: [0, -4, 0] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
        >
          <span className="text-[22px] font-bold text-white">Æ</span>
        </motion.div>

        <h1 className="text-[17px] font-semibold tracking-tight text-ink">Aether GUI</h1>
        <p className="text-[11.5px] text-ink-dim">by NetRepublic</p>
        <p className="tnum mt-1 text-[10.5px] text-ink-faint">
          GUI v1.0.0 · core v{coreVersion || "…"}
        </p>

        <div className="mt-3 flex items-center gap-1.5">
          <span className="text-[12px] text-ink-dim">Made by Amirreza</span>
          <motion.span
            animate={{ scale: [1, 1.22, 1] }}
            transition={{ duration: 1.4, repeat: Infinity, ease: "easeInOut" }}
          >
            <Heart size={12} className="fill-danger text-danger" />
          </motion.span>
        </div>
      </motion.div>

      <motion.div variants={staggerChild} className="glass p-3.5">
        <p className="text-[11.5px] font-medium text-ink">Powered by the Aether core</p>
        <p className="mt-1.5 text-[10.5px] leading-relaxed text-ink-dim">
          The tunnelling engine — MASQUE, WireGuard, gool, endpoint discovery and obfuscation — is
          built by <span className="text-ink">CluvexStudio</span>. This app is only a Windows front
          end for it. Huge thanks to them for building and open-sourcing the hard part. 🙏
        </p>
        <LinkRow
          icon={<Github size={13} />}
          label="github.com/CluvexStudio/Aether"
          onClick={() => void api.openUrl(GITHUB_UPSTREAM)}
        />
      </motion.div>

      <motion.div variants={staggerChild} className="glass p-3.5">
        <p className="text-[11.5px] font-medium text-ink">NetRepublic</p>
        <p className="mt-1.5 text-[10.5px] leading-relaxed text-ink-dim">
          Tools, tech and utilities. Join the channel for updates and new releases.
        </p>
        <LinkRow
          icon={<Send size={13} />}
          label="t.me/net_republic"
          onClick={() => void api.openUrl(TELEGRAM_CHANNEL)}
        />
      </motion.div>

      <motion.div variants={staggerChild} className="glass p-3.5">
        <p className="text-[11.5px] font-medium text-ink">Licence & credits</p>
        <ul className="mt-1.5 flex flex-col gap-1 text-[10.5px] leading-relaxed text-ink-faint">
          <li>Aether core © CluvexStudio — AGPL-3.0</li>
          <li>This GUI — AGPL-3.0, source published</li>
          <li>MASQUE built on Cloudflare Quiche</li>
          <li>TUN device via Wintun (WireGuard LLC)</li>
        </ul>
      </motion.div>

      <motion.p variants={staggerChild} className="pt-1 text-center text-[10px] text-ink-faint">
        Aether is a trademark of CluvexStudio. This is an independent front end.
      </motion.p>
    </motion.div>
  );
}

function LinkRow({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="mt-2.5 flex w-full items-center gap-2 rounded-lg border border-hairline bg-black/25 px-2.5 py-2 text-left outline-none transition-colors hover:border-white/15 hover:bg-white/[0.04]"
    >
      <span className="shrink-0 text-ink-dim">{icon}</span>
      <span className="flex-1 truncate text-[11px] text-ink-dim">{label}</span>
      <ExternalLink size={11} className="shrink-0 text-ink-faint" />
    </button>
  );
}
