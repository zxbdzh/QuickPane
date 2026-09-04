import type { Transition, Variants } from "motion/react";

/**
 * 全部动效参数收敛在此：spring 为主，交互反馈硬上限 300ms。
 * 组件只引用这里的 variants / transition，不零散写 duration。
 */

/** Fluent 风格 ease-out 曲线（保留给简单 tween） */
export const EASE_OUT: [number, number, number, number] = [0.16, 1, 0.3, 1];

/** 弹簧预设：交互 / 浮层 / 列表级联 / layoutId 指示条 / 卡片 */
export const SPRING = {
  interaction: { type: "spring", stiffness: 480, damping: 32 },
  overlay: { type: "spring", stiffness: 420, damping: 34 },
  list: { type: "spring", stiffness: 350, damping: 30 },
  layout: { type: "spring", stiffness: 480, damping: 38 },
  card: { type: "spring", stiffness: 320, damping: 28 },
} satisfies Record<string, Transition>;

/** layoutId 指示条（活动标签下划线）专用 */
export const INDICATOR_TRANSITION: Transition = SPRING.layout;

/** 页面切换：新页自下浮入，旧页向上退出（方向感） */
export const pageFade: Variants = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0, transition: { duration: 0.16, ease: EASE_OUT } },
  exit: { opacity: 0, y: -6, transition: { duration: 0.1, ease: "easeIn" } },
};

/** 非 Radix 浮层（标签面板 / 地址建议下拉）：弹簧入场 + 快速退场 */
export const overlay: Variants = {
  initial: { opacity: 0, scale: 0.96, y: -4 },
  animate: {
    opacity: 1,
    scale: 1,
    y: 0,
    transition: SPRING.overlay,
  },
  exit: {
    opacity: 0,
    scale: 0.98,
    y: -2,
    transition: { duration: 0.09, ease: "easeIn" },
  },
};

/** 错误横幅：顶部弹簧滑入，快速滑出 */
export const bannerSlide: Variants = {
  initial: { opacity: 0, y: -14, scale: 0.98 },
  animate: { opacity: 1, y: 0, scale: 1, transition: SPRING.overlay },
  exit: {
    opacity: 0,
    y: -10,
    scale: 0.98,
    transition: { duration: 0.12, ease: "easeIn" },
  },
};

/** 列表项级联入场（新标签页快捷站点） */
export const listItem: Variants = {
  initial: { opacity: 0, y: 10 },
  animate: (index: number) => ({
    opacity: 1,
    y: 0,
    transition: { ...SPRING.list, delay: Math.min(index * 0.03, 0.12) },
  }),
};

/** 锁屏卡片入场 */
export const lockCard: Variants = {
  initial: { opacity: 0, y: 14, scale: 0.98 },
  animate: { opacity: 1, y: 0, scale: 1, transition: SPRING.card },
};

/** 窗口呼出 reveal：根节点只动 opacity（不改变 fixed 包含块） */
export const revealRoot: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.12 } },
};

/** 窗口呼出 reveal：内容区上浮，提供方向感 */
export const revealContent: Variants = {
  hidden: { opacity: 0, y: -6 },
  visible: { opacity: 1, y: 0, transition: { ...SPRING.overlay, delay: 0.05 } },
};

/** 标签出入场：缩放弹出 / 宽度收窄驱动兄弟补位 */
export const tabMotion: Variants = {
  initial: { opacity: 0, scale: 0.92 },
  animate: { opacity: 1, scale: 1, transition: SPRING.list },
  exit: {
    opacity: 0,
    width: 0,
    minWidth: 0,
    transition: { duration: 0.15, ease: "easeIn" },
  },
};

/** 密码错误抖动：150ms，配合 useAnimationControls 播放 */
export const shakeKeyframes = { x: [0, -10, 10, -6, 6, 0] };
export const shakeTransition: Transition = { duration: 0.15, ease: "easeOut" };
