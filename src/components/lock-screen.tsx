import { Eye, EyeOff, KeyRound, Lock } from "lucide-react";
import { useAnimationControls, motion } from "motion/react";
import type { FormEvent } from "react";
import { useCallback, useState } from "react";

import { api } from "../api";
import type { AppSnapshot } from "../types";
import { lockCard, shakeKeyframes, shakeTransition } from "../lib/motion";
import { Button } from "./ui/button";
import { Input } from "./ui/input";

function LockScreen({ snapshot, applySnapshot, run }: {
  snapshot: AppSnapshot;
  applySnapshot: (value: AppSnapshot) => void;
  run: <T>(action: () => Promise<T>) => Promise<T | undefined>;
}) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [reveal, setReveal] = useState(false);
  const shakeControls = useAnimationControls();
  const hasPassword = snapshot.hasPassword;
  const firstSetup = snapshot.firstRun && !hasPassword;

  const shake = useCallback(() => {
    void shakeControls.start({ ...shakeKeyframes, transition: shakeTransition });
  }, [shakeControls]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (firstSetup) {
      if (password !== confirm) {
        shake();
        return;
      }
      const next = await run(() => api.setPassword(password));
      if (next) applySnapshot(next);
      else shake();
    } else {
      const next = await run(() => api.unlock(password));
      if (next) applySnapshot(next);
      else {
        setPassword("");
        shake();
      }
    }
  };

  return (
    <div className="relative grid h-full min-h-0 place-items-center overflow-hidden bg-background px-6">
      <div aria-hidden className="pointer-events-none absolute -top-40 -left-28 size-96 rounded-full bg-primary/10 blur-3xl" />
      <div aria-hidden className="pointer-events-none absolute -right-32 -bottom-44 size-[28rem] rounded-full bg-primary/8 blur-3xl" />

      <motion.div variants={lockCard} initial="initial" animate="animate" className="relative w-[min(390px,100%)]">
        <motion.div
          animate={shakeControls}
          className="rounded-lg border bg-surface/90 p-7 text-center shadow-popover backdrop-blur-xl"
        >
          <div className="mx-auto mb-4 grid size-12 place-items-center rounded-lg bg-soft text-on-soft">
            {firstSetup ? <KeyRound className="size-6" /> : <Lock className="size-6" />}
          </div>
          <h1 className="text-xl font-semibold">{firstSetup ? "保护 QuickPane" : "QuickPane 已锁定"}</h1>
          <p className="mx-auto mt-2 mb-5 max-w-[310px] text-[13px] leading-5 text-muted-foreground">
            {firstSetup
              ? "应用密码可选。启用后，冷启动和 Windows 锁屏后需要验证。"
              : "输入应用密码以恢复上次会话。"}
          </p>
          <form className="flex flex-col gap-2.5" onSubmit={submit}>
            <div className="relative">
              <label htmlFor="lock-password" className="sr-only">{firstSetup ? "设置应用密码" : "应用密码"}</label>
              <Input
                id="lock-password"
                autoFocus
                type={reveal ? "text" : "password"}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                placeholder={firstSetup ? "设置应用密码" : "应用密码"}
                className="h-10 bg-surface pr-10"
              />
              <button
                type="button"
                onClick={() => setReveal(!reveal)}
                aria-label="显示或隐藏密码"
                className="absolute top-1/2 right-1 grid size-8 -translate-y-1/2 place-items-center rounded-sm text-muted-foreground transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
              >
                {reveal ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
              </button>
            </div>
            {firstSetup ? (
              <div>
                <label htmlFor="lock-confirm" className="sr-only">再次输入密码</label>
                <Input
                  id="lock-confirm"
                  type={reveal ? "text" : "password"}
                  value={confirm}
                  onChange={(event) => setConfirm(event.target.value)}
                  placeholder="再次输入密码"
                  className="h-10 bg-surface"
                />
              </div>
            ) : null}
            {firstSetup && confirm && password !== confirm ? (
              <p className="text-left text-xs text-destructive">两次输入的密码不一致</p>
            ) : null}
            <Button
              type="submit"
              className="mt-1 h-10 w-full"
              disabled={password.length < 4 || (firstSetup && password !== confirm)}
            >
              {firstSetup ? "启用并继续" : "解锁"}
            </Button>
          </form>
          {firstSetup ? (
            <Button
              variant="ghost"
              size="sm"
              className="mx-auto mt-3 text-muted-foreground hover:text-foreground"
              onClick={() => void run(api.skipPasswordSetup).then((next) => { if (next) applySnapshot(next); })}
            >
              暂不设置
            </Button>
          ) : (
            <p className="mt-4 text-xs text-faint">忘记密码只能清除全部 QuickPane 数据后重置。</p>
          )}
        </motion.div>
      </motion.div>
    </div>
  );
}

export { LockScreen };
