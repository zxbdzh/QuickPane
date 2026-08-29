import type { ComponentProps } from "react";

import { cn } from "../../lib/utils";

function Card({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      className={cn("rounded-lg border bg-card text-card-foreground", className)}
      {...props}
    />
  );
}

function CardHeader({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("flex flex-col gap-1", className)} {...props} />;
}

function CardTitle({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("text-base font-semibold leading-none", className)} {...props} />;
}

function CardDescription({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("text-xs leading-5 text-muted-foreground", className)} {...props} />;
}

function CardContent({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("", className)} {...props} />;
}

export { Card, CardHeader, CardTitle, CardDescription, CardContent };
