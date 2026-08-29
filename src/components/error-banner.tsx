import { CircleAlert } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import { bannerSlide } from "../lib/motion";
import { Toast, ToastClose, ToastDescription, ToastProvider, ToastViewport } from "./ui/toast";

/** 错误横幅：低饱和红背景，顶部滑入滑出（基于 Radix Toast + motion） */
function ErrorBanner({ error, onDismiss }: { error: string | null; onDismiss: () => void }) {
  return (
    <ToastProvider duration={Infinity}>
      <ToastViewport />
      <AnimatePresence>
        {error ? (
          <Toast
            key="error-banner"
            variant="banner"
            className="pointer-events-auto outline-none"
            open
            onOpenChange={(open) => { if (!open) onDismiss(); }}
          >
            <motion.div
              variants={bannerSlide}
              initial="initial"
              animate="animate"
              exit="exit"
              className="flex w-full items-center gap-2"
            >
              <CircleAlert className="size-4 shrink-0" />
              <ToastDescription>{error}</ToastDescription>
              <ToastClose aria-label="关闭错误" onClick={onDismiss} />
            </motion.div>
          </Toast>
        ) : null}
      </AnimatePresence>
    </ToastProvider>
  );
}

export { ErrorBanner };
