import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Input } from "@repo/ui/components/input";
import { createFileRoute } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { RESET } from "jotai/utils";
import { useState } from "react";
import { crawlResultAtom } from "../atoms/crawl-result";
import { useSettings } from "../atoms/settings";
import { AnalysisStatus } from "../components/analysis-status";
import { IssueCategoryDetail } from "../components/display/issue-category-detail";
import { IssueCategoryOverview } from "../components/display/issue-category-overview";
import { LinkDisplay } from "../components/display/link-display";
import { commands } from "../generated/bindings";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useAtom(crawlResultAtom);

  const { baseUrl, setBaseUrl } = useSettings();

  const analyzeSeo = async () => {
    try {
      setLoading(true);
      setResult(RESET);
      const analysis = await commands.analyzeUrlSeo();
      console.log("Analysis", analysis);
      if (analysis.status === "ok") {
        setResult(analysis.data);
      }
    } catch (error) {
      console.error("Error analyzing SEO:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto flex max-w-4xl flex-col gap-4 p-4">
      {/* <SettingsCard collapsible /> */}
      <Card>
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
      </Card>
    </div>
  );
}
