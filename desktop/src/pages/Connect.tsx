import { motion } from "framer-motion";
import { PowerButton } from "../components/PowerButton";
import { ServerChip } from "../components/ServerChip";
import { SpeedCard } from "../components/SpeedCard";
import { StatusText } from "../components/StatusText";
import { staggerParent } from "../lib/motion";
import { useStore } from "../lib/store";

export function ConnectPage() {
  const phase = useStore((s) => s.phase);
  const detail = useStore((s) => s.detail);
  const elapsed = useStore((s) => s.elapsed);
  const peer = useStore((s) => s.peer);
  const samples = useStore((s) => s.samples);
  const downBps = useStore((s) => s.downBps);
  const upBps = useStore((s) => s.upBps);
  const mode = useStore((s) => s.settings.mode);
  const toggle = useStore((s) => s.toggle);

  const live = phase === "connected";

  return (
    <div className="flex h-full flex-col items-center px-5 pb-2">
      <div className="flex flex-1 flex-col items-center justify-center">
        <PowerButton phase={phase} onClick={() => void toggle()} />
        <StatusText phase={phase} detail={detail} elapsed={elapsed} />
      </div>

      {/* Reserve the space so the button never jumps when a tunnel comes up. */}
      <div className="flex min-h-[150px] w-full flex-col justify-end gap-2 pb-1">
        {live && (
          <motion.div
            variants={staggerParent}
            initial="initial"
            animate="animate"
            className="flex w-full flex-col gap-2"
          >
            <div className="flex gap-2">
              <SpeedCard direction="down" bps={downBps} history={samples.map((s) => s.down)} />
              <SpeedCard direction="up" bps={upBps} history={samples.map((s) => s.up)} />
            </div>
            {peer && <ServerChip peer={peer} mode={mode} />}
          </motion.div>
        )}
      </div>
    </div>
  );
}
