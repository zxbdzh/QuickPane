import * as ToastPrimitive from "@radix-ui/react-toast";
import { cva, type VariantProps } from "class-variance-authority";
import { X } from "lucide-react";
import type { ComponentProps } from "react";

import { cn } from "../../lib/utils";

/**
 * 基于 Radix Toast 原语的 shadcn Toast。
 * 出入场动画由使用方（error-banner.tsx）经 motion 控制。
 */

const toastVariants = cva(
  "pointer-events-auto relative flex w-full items-center gap-2 overflow-hidden rounded-md border px-3 py-2.5 text-sm shadow-popover transition-colors",
  {
    variants: {
      variant: {
        default: "border bg-popover text-popover-foreground",
        banner:
          "border-destructive-border bg-destructive-soft text-destructive",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

function ToastProvider({ ...props }: ComponentProps<typeof ToastPrimitive.Provider>) {
  return <ToastPrimitive.Provider swipeDirection="up" {...props} />;
}

function ToastViewport({
  className,
  ...props
}: ComponentProps<typeof ToastPrimitive.Viewport>) {
  return (
    <ToastPrimitive.Viewport
      className={cn(
        "fixed top-[92px] left-1/2 z-[100] flex w-[min(560px,calc(100vw-28px))] -translate-x-1/2 flex-col items-center outline-none",
        className,
      )}
      {...props}
    />
  );
}

function Toast({ className, variant, ...props }: ComponentProps<typeof ToastPrimitive.Root> &
  VariantProps<typeof toastVariants>) {
  return <ToastPrimitive.Root className={cn(toastVariants({ variant }), className)} {...props} />;
}

function ToastTitle({ className, ...props }: ComponentProps<typeof ToastPrimitive.Title>) {
  return <ToastPrimitive.Title className={cn("text-sm font-medium", className)} {...props} />;
}

function ToastDescription({
  className,
  ...props
}: ComponentProps<typeof ToastPrimitive.Description>) {
  return (
    <ToastPrimitive.Description
      className={cn("min-w-0 flex-1 break-all text-[13px] leading-5", className)}
      {...props}
    />
  );
}

function ToastClose({ className, ...props }: ComponentProps<typeof ToastPrimitive.Close>) {
  return (
    <ToastPrimitive.Close
      className={cn(
        "pointer-events-auto -mr-1 flex size-6 shrink-0 items-center justify-center rounded-sm transition-colors hover:bg-destructive/10 focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none",
        className,
      )}
      {...props}
    >
      <X className="size-3.5" />
    </ToastPrimitive.Close>
  );
}

export { ToastProvider, ToastViewport, Toast, ToastTitle, ToastDescription, ToastClose, toastVariants };
