import { AnimatePresence, motion } from "framer-motion";
import { useEffect } from "react";
import { Backdrop } from "./components/Backdrop";
import { BottomNav } from "./components/BottomNav";
import { TitleBar } from "./components/TitleBar";
import { Toast } from "./components/Toast";
import { swapPage } from "./lib/motion";
import { useStore } from "./lib/store";
import { AboutPage } from "./pages/About";
import { ConnectPage } from "./pages/Connect";
import { SettingsPage } from "./pages/Settings";
import { StatsPage } from "./pages/Stats";

export default function App() {
  const page = useStore((s) => s.page);
  const setPage = useStore((s) => s.setPage);
  const phase = useStore((s) => s.phase);
  const toast = useStore((s) => s.toast);
  const dismissToast = useStore((s) => s.dismissToast);
  const init = useStore((s) => s.init);
  const tick = useStore((s) => s.tick);

  useEffect(() => {
    void init();
  }, [init]);

  // Single 1s heartbeat drives the uptime label; the backend never sends it.
  useEffect(() => {
    const id = window.setInterval(tick, 1000);
    return () => window.clearInterval(id);
  }, [tick]);

  return (
    <div className="relative flex h-full flex-col overflow-hidden bg-base">
      <Backdrop live={phase === "connected"} />

      <div className="relative z-10 flex h-full flex-col">
        <TitleBar />

        <main className="relative flex-1 overflow-hidden">
          <AnimatePresence mode="wait" initial={false}>
            <motion.div
              key={page}
              variants={swapPage}
              initial="initial"
              animate="animate"
              exit="exit"
              transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
              className="absolute inset-0"
            >
              {page === "connect" && <ConnectPage />}
              {page === "settings" && <SettingsPage />}
              {page === "stats" && <StatsPage />}
              {page === "about" && <AboutPage />}
            </motion.div>
          </AnimatePresence>

          <Toast toast={toast} onDismiss={dismissToast} />
        </main>

        <BottomNav page={page} onChange={setPage} />
      </div>
    </div>
  );
}
