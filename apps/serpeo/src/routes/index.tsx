import { Button } from "@repo/ui/components/button";
import { Label } from "@repo/ui/components/label";
import { ShineBorder } from "@repo/ui/components/shine-border";
import { WavyBackground } from "@repo/ui/components/wavy-background";
import { Link, createFileRoute } from "@tanstack/react-router";
import { Settings } from "lucide-react";
import { useEffect } from "react";
import { AnalyzeSeoInput } from "../components/analyze-seo-input";
export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  useEffect(() => {
    document.body.classList.add("h-dvh");
    document.body.classList.add("overflow-hidden");
    return () => {
      document.body.classList.remove("h-dvh");
      document.body.classList.remove("overflow-hidden");
    };
  }, []);

  return (
    <WavyBackground className="flex max-h-dvh max-w-dvh items-center justify-center overflow-hidden [view-transition-name:main-content]">
      <div className="relative rounded-md bg-background p-4 shadow-lg">
        <ShineBorder borderWidth={2} shineColor={"chart"} />
        <Button
          className="fixed right-4 bottom-4"
          variant="outline"
          size="icon"
          asChild
          aria-label="Settings"
        >
          <Link to="/settings" viewTransition>
            <Settings />
          </Link>
        </Button>
        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label>Analyze Your Website</Label>
            <AnalyzeSeoInput />
          </div>
        </div>
      </div>
    </WavyBackground>
  );
}
