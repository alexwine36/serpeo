import { Badge } from "@repo/ui/components/badge";
import { Button } from "@repo/ui/components/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@repo/ui/components/dialog";
import { Input } from "@repo/ui/components/input";
import { Label } from "@repo/ui/components/label";
import { ShineBorder } from "@repo/ui/components/shine-border";
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
  const [baseUrl, setBaseUrl] = useState("");

  console.log(sites);
  const analyzeSeo = async () => {
    try {
      setLoading(true);
      setResult(RESET);
      const analysis = await commands.analyzeUrlSeo(baseUrl);
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
    // <WavyBackground
    //   // rangeY={200}
    //   //     // particleCount={500}
    //   //     // baseHue={120}
    //   //     baseRadius={3}
    //   //     rangeRadius={2}
    //   className="max-w-screen overflow-x-hidden"
    // >
    <WavyBackground className="flex max-h-dvh max-w-dvh items-center justify-center overflow-hidden [view-transition-name:warp]">
      {/* <WavyBackground className="max-h-full overflow-hidden"> */}
      <div className="relative rounded-md bg-background p-4 shadow-lg">
        {/* <SettingsCard collapsible /> */}
        <ShineBorder
          borderWidth={2}
          // className="bg-background"
          shineColor={"chart"}
          // shineColor={["#A07CFE", "#FE8FB5", "#FFBE7B"]}
        />
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
          <div className="space-4 grid w-[calc(100vw-4rem)] grid-cols-1 gap-4 sm:w-sm md:w-md md:grid-cols-[1fr_auto]">
            <div className="flex flex-col gap-2">
              <Label>Analyze Your Website</Label>
              <Input
                type="url"
                placeholder="https://"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                className="flex-1"
              />
            </div>
            <div className="flex items-end">
              <Button
                className="min-w-24"
                onClick={analyzeSeo}
                disabled={loading || !baseUrl}
              >
                {loading ? "Analyzing..." : "Analyze"}
              </Button>
            </div>
          </div>
          {sites && (
            <div className="flex w-full flex-row flex-wrap gap-2">
              {sites
                ?.filter((_, idx) => idx < 5)
                .map((site) => (
                  <Badge
                    key={site.id}
                    onClick={() => setBaseUrl(site.url)}
                    className="cursor-pointer"
                  >
                    {site.url}
                  </Badge>
                ))}
            </div>
          )}
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

        {/* {result.total_pages > 0 && (
            <div className="mt-8 space-y-6">
              <LinkDisplay />
              <IssueCategoryOverview />
              <IssueCategoryDetail />
            </div>
          )} */}

        {/* <Card>
            <CardHeader>
              <CardTitle>SEO Analysis Tool</CardTitle>
              <CardDescription>
                Run a full SEO analysis on the website configured in settings.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-4 grid grid-cols-1 gap-4 sm:grid-cols-[1fr_auto]">
                <Input
                  type="url"
                  placeholder="Enter website URL..."
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  className="flex-1"
                />
                <Button
                  className="min-w-24"
                  onClick={analyzeSeo}
                  disabled={loading || !baseUrl}
                >
                  {loading ? "Analyzing..." : "Analyze"}
                </Button>
              </div>

              <AnalysisStatus />
              {result.total_pages > 0 && (
                <div className="mt-8 space-y-6">
                  <LinkDisplay />
                  <IssueCategoryOverview />
                  <IssueCategoryDetail />
                </div>
              )}
            </CardContent>
          </Card> */}
      </div>
      {/* </WavyBackground> */}
    </WavyBackground>

    // </WavyBackground>
  );
}
