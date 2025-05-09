"use client";

import { cn } from "@repo/ui/lib/utils";
import type * as React from "react";
import { useMemo } from "react";

interface ShineBorderProps extends React.HTMLAttributes<HTMLDivElement> {
  /**
   * Width of the border in pixels
   * @default 1
   */
  borderWidth?: number;
  /**
   * Duration of the animation in seconds
   * @default 14
   */
  duration?: number;
  /**
   * Color of the border, can be a single color or an array of colors
   * @default "#000000"
   */
  shineColor?: string | string[] | "chart";
}

/**
 * Shine Border
 *
 * An animated background border effect component with configurable properties.
 */
export function ShineBorder({
  borderWidth = 1,
  duration = 14,
  shineColor = "chart",
  className,
  style,
  ...props
}: ShineBorderProps) {
  const styles = getComputedStyle(document.documentElement);
  const shineColors: string[] = useMemo(() => {
    if (shineColor === "chart") {
      return [
        styles.getPropertyValue("--chart-1"),
        styles.getPropertyValue("--chart-2"),
        styles.getPropertyValue("--chart-3"),
        styles.getPropertyValue("--chart-4"),
        styles.getPropertyValue("--chart-5"),
      ];
    }
    if (!Array.isArray(shineColor)) {
      return [shineColor];
    }
    return shineColor;
  }, [shineColor, styles]);
  return (
    <div
      style={
        {
          "--border-width": `${borderWidth}px`,
          "--duration": `${duration}s`,
          backgroundImage: `radial-gradient(transparent,transparent, ${shineColors.join(
            ","
          )},transparent,transparent)`,
          backgroundSize: "300% 300%",
          mask: "linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)",
          WebkitMask:
            "linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)",
          WebkitMaskComposite: "xor",
          maskComposite: "exclude",
          padding: "var(--border-width)",
          ...style,
        } as React.CSSProperties
      }
      className={cn(
        "pointer-events-none absolute inset-0 size-full rounded-[inherit] will-change-[background-position] motion-safe:animate-shine",
        className
      )}
      {...props}
    />
  );
}
