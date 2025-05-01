import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { AnalysisStatus } from "../components/analysis-status";
import { SettingsCard } from "../components/settings/card";
import { type CrawlResult, commands } from "../generated/bindings";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<CrawlResult | null>(null);

  const analyzeSeo = async () => {
    try {
      setLoading(true);
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

  // const charts = useMemo((): {
  //   label: string;
  //   value: number;
  // }[] => {
  //   if (!result?.lighthouse_metrics) {
  //     return [];
  //   }
  //   const {
  //     performance_score,
  //     best_practices_score,
  //     seo_score,
  //     accessibility_score,
  //   } = result.lighthouse_metrics;
  //   return [
  //     {
  //       label: "Performance",
  //       value: performance_score,
  //     },
  //     {
  //       label: "SEO",
  //       value: seo_score,
  //     },
  //     {
  //       label: "Best Practices",
  //       value: best_practices_score,
  //     },
  //     {
  //       label: "Accessibility",
  //       value: accessibility_score,
  //     },
  //   ];
  // }, [result]);
  return (
    <div className="container mx-auto flex max-w-4xl flex-col gap-4 p-4">
      <SettingsCard collapsible />
      <Card>
        <CardHeader>
          <CardTitle>SEO Analysis Tool</CardTitle>
          <CardDescription>
            Run a full SEO analysis on the website configured in settings.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <Button onClick={analyzeSeo} disabled={loading}>
              {loading ? "Analyzing..." : "Analyze"}
            </Button>
          </div>

          <AnalysisStatus />
          {result && (
            <div className="mt-8 space-y-6">
              <div>
                <h2 className="font-semibold text-lg">Total Pages</h2>
                <p className="font-bold text-2xl">{result.total_pages}</p>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
