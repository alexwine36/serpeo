import { type VariantProps, cva } from "class-variance-authority";
import { cn } from "../../../lib/utils.js";

const spinnerVariants = cva("", {
  variants: {
    size: {
      sm: "h-6 w-6",
      md: "h-10 w-10",
      lg: "h-24 w-24",
    },
  },
  defaultVariants: {
    size: "sm",
  },
});

export interface SwirlingEffectSpinnerProps
  extends React.HTMLAttributes<HTMLOrSVGElement>,
    VariantProps<typeof spinnerVariants> {
  loading?: boolean;
  asChild?: boolean;
}

export const SwirlingEffectSpinner = ({
  className,
  size,
  ...props
}: SwirlingEffectSpinnerProps) => {
  return (
    <>
      <style>
        {`@keyframes spin {
            to {
              transform: rotate(360deg);
            }
          }
        
          @keyframes spin2 {
            0% {
              stroke-dasharray: 1, 800;
              stroke-dashoffset: 0;
            }
            50% {
              stroke-dasharray: 400, 400;
              stroke-dashoffset: -200px;
            }
            100% {
              stroke-dasharray: 800, 1;
              stroke-dashoffset: -800px;
            }
          }
        
          .spin2 {
            transform-origin: center;
            animation: spin2 1.5s ease-in-out infinite,
              spin 2s linear infinite;
            animation-direction: alternate;
          }`}
      </style>

      <svg
        viewBox="0 0 800 800"
        className={cn(spinnerVariants({ size }), className)}
        xmlns="http://www.w3.org/2000/svg"
        aria-label="Loading..."
        {...props}
      >
        <title>Loading...</title>
        <circle
          className="spin2 stroke-primary"
          cx="400"
          cy="400"
          fill="none"
          r="200"
          strokeWidth="50"
          strokeDasharray="700 1400"
          strokeLinecap="round"
        />
      </svg>
    </>
  );
};
