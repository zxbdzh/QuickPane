import type { Transition, Variants } from "motion/react";

/**
 * 全部动效参数收敛在此：100-150ms ease-out，最长不超过 300ms。
 * 组件只引用这里的 variants / transition，不零散写 duration。
 */

/** Fluent 风格 ease-out 曲线 */
export const EASE_OUT: [number, number, number, number] = [0.16, 1, 0.3, 1];

/** 页面切换：内容区淡入上浮 */
export const pageFade: Variants = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0, transition: { duration: 0.15, ease: EASE_OUT } },
  exit: { opacity: 0, y: 4, transition: { duration: 0.1, ease: "easeIn" } },
};

/** 浮层 / 菜单入场：轻微缩放 + 下移淡入 */
export const overlayIn: Variants = {
  initial: { opacity: 0, scale: 0.97, y: -4 },
  animate: { opacity: 1, scale: 1, y: 0, transition: { duration: 0.12, ease: EASE_OUT } },
};

/** 错误横幅：顶部滑入滑出 */
export const bannerSlide: Variants = {
  initial: { opacity: 0, y: -14, scale: 0.98 },
  animate: { opacity: 1, y: 0, scale: 1, transition: { duration: 0.15, ease: EASE_OUT } },
  exit: { opacity: 0, y: -10, scale: 0.98, transition: { duration: 0.12, ease: "easeIn" } },
};

/** 列表项级联入场（新标签页快捷站点） */
export const listItem: Variants = {
  initial: { opacity: 0, y: 10 },
  animate: (index: number) => ({
    opacity: 1,
    y: 0,
    transition: { duration: 0.15, ease: EASE_OUT, delay: Math.min(index * 0.02, 0.1) },
  }),
};

/** 锁屏卡片入场 */
export const lockCard: Variants = {
  initial: { opacity: 0, y: 14, scale: 0.98 },
  animate: { opacity: 1, y: 0, scale: 1, transition: { duration: 0.2, ease: EASE_OUT } },
};

/** 密码错误抖动：150ms，配合 useAnimationControls 播放 */
export const shakeKeyframes = { x: [0, -10, 10, -6, 6, 0] };
export const shakeTransition: Transition = { duration: 0.15, ease: "easeOut" };
