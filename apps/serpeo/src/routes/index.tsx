import { Button } from "@repo/ui/components/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@repo/ui/components/dialog";
import { Label } from "@repo/ui/components/label";
import { ShineBorder } from "@repo/ui/components/shine-border";
import { UrlInput } from "@repo/ui/components/url-input";
import { WavyBackground } from "@repo/ui/components/wavy-background";
import { Link, createFileRoute, useNavigate } from "@tanstack/react-router";
import { useSetAtom } from "jotai";
import { RESET } from "jotai/utils";
import { Settings } from "lucide-react";
import { useEffect, useState } from "react";
import { crawlResultAtom } from "../atoms/crawl-result";
import { AnalysisStatus } from "../components/analysis-status";
import { commands } from "../generated/bindings";
import { useSitesQuery } from "../queries/sites";
export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [loading, setLoading] = useState(false);
  const setResult = useSetAtom(crawlResultAtom);
  const navigate = useNavigate();
  const { data: sites } = useSitesQuery();

  const analyzeSeo = async (url: string) => {
    try {
      setLoading(true);
      setResult(RESET);
      const analysis = await commands.analyzeUrlSeo(url);
      console.log("Analysis", analysis);
      if (analysis.status === "ok") {
        setResult(analysis.data);
        navigate({ to: "/analysis", viewTransition: true });
      }
    } catch (error) {
      console.error("Error analyzing SEO:", error);
    } finally {
      setLoading(false);
    }
  };

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
            <UrlInput
              className="md:w-sm lg:w-md"
              onSubmit={(url) => {
                analyzeSeo(url);
              }}
              previousUrls={sites?.map(({ site }) => site.url)}
            />
          </div>
        </div>

        <Dialog open={loading}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Run Status</DialogTitle>
            </DialogHeader>
            <div>
              <AnalysisStatus />
            </div>
          </DialogContent>
        </Dialog>
      </div>
    </WavyBackground>
  );
}
