import { Slot } from "@radix-ui/react-slot";
import { type VariantProps, cva } from "class-variance-authority";
import type * as React from "react";

import { cn } from "@repo/ui/lib/utils";

const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden whitespace-nowrap rounded-md border px-2 py-0.5 font-medium text-xs transition-[color,box-shadow] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-primary text-primary-foreground [a&]:hover:bg-primary/90",
        secondary:
          "border-transparent bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90",
        destructive:
          "border-transparent bg-destructive text-white focus-visible:ring-destructive/20 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40 [a&]:hover:bg-destructive/90",
        outline:
          "text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
      },
      outlineColor: {
        default: "",

        orange:
          "border-orange-500 bg-orange-100/25 text-orange-700 dark:bg-orange-900 dark:text-orange-100",
        cyan: "border-cyan-500 bg-cyan-100/25 text-cyan-700 dark:bg-cyan-900 dark:text-cyan-100",
        blue: "border-blue-500 bg-blue-100/25 text-blue-700 dark:bg-blue-900 dark:text-blue-100",
        red: "border-red-500 bg-red-100/25 text-red-700 dark:bg-red-900 dark:text-red-100",
        green:
          "border-green-500 bg-green-100/25 text-green-700 dark:bg-green-900 dark:text-green-100",
        slate:
          "border-slate-500 bg-slate-100/25 text-slate-700 dark:bg-slate-900 dark:text-slate-100",
        purple:
          "border-purple-500 bg-purple-100/25 text-purple-700 dark:bg-purple-900 dark:text-purple-100",
        pink: "border-pink-500 bg-pink-100/25 text-pink-700 dark:bg-pink-900 dark:text-pink-100",
        yellow:
          "border-yellow-500 bg-yellow-100/25 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-100",
        indigo:
          "border-indigo-500 bg-indigo-100/25 text-indigo-700 dark:bg-indigo-900 dark:text-indigo-100",
        teal: "border-teal-500 bg-teal-100/25 text-teal-700 dark:bg-teal-900 dark:text-teal-100",
        violet:
          "border-violet-500 bg-violet-100/25 text-violet-700 dark:bg-violet-900 dark:text-violet-100",
      },
    },
    defaultVariants: {
      variant: "default",
      outlineColor: "default",
    },
  }
);

export type BadgeProps = React.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean };

function Badge({
  className,
  variant,
  outlineColor,
  asChild = false,
  ...props
}: BadgeProps) {
  const Comp = asChild ? Slot : "span";

  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant, outlineColor }), className)}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
