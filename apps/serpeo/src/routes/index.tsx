import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Input } from "@repo/ui/components/input";
import { RadialChart } from "@repo/ui/custom/radial-chart";
import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { type SeoAnalysis, commands } from "../bindings";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [url, setUrl] = useState("https://stem-programs.newspacenexus.org/");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<SeoAnalysis | null>(null);

  const analyzeSeo = async () => {
    try {
      setLoading(true);
      const analysis = await commands.analyzeSeo(url);
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

  const charts = useMemo((): {
    label: string;
    value: number;
  }[] => {
    if (!result?.lighthouse_metrics) {
      return [];
    }
    const {
      performance_score,
      best_practices_score,
      seo_score,
      accessibility_score,
    } = result.lighthouse_metrics;
    return [
      {
        label: "Performance",
        value: performance_score,
      },
      {
        label: "SEO",
        value: seo_score,
      },
      {
        label: "Best Practices",
        value: best_practices_score,
      },
      {
        label: "Accessibility",
        value: accessibility_score,
      },
    ];
  }, [result]);
  return (
    <div className="container mx-auto max-w-4xl p-4">
      <Card>
        <CardHeader>
          <CardTitle>SEO Analysis Tool</CardTitle>
          <CardDescription>
            Enter a URL to analyze its SEO performance and get detailed insights
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <Input
              type="url"
              placeholder="Enter website URL..."
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              className="flex-1"
            />
            <Button onClick={analyzeSeo} disabled={loading || !url}>
              {loading ? "Analyzing..." : "Analyze"}
            </Button>
          </div>

          {result && (
            <div className="mt-8 space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Lighthouse</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid h-full grid-cols-2 gap-2">
                    {charts.map((chart) => {
                      return (
                        <div key={chart.label} className="h-full">
                          <RadialChart
                            title={chart.label}
                            value={chart.value}
                          />
                        </div>
                      );
                    })}

                    {/* <p>
                      <strong>Title:</strong> {result.meta_tags.title}
                    </p> */}
                    {/* <p>
                      <strong>Description:</strong>{" "}
                      {result.meta_tags.description}
                    </p>
                    <p>
                      <strong>Keywords:</strong>{" "}
                      {result.meta_tags.keywords.join(", ")}
                    </p> */}
                  </div>
                </CardContent>
              </Card>
              <Card>
                <CardHeader>
                  <CardTitle>Meta Tags Analysis</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    <p>
                      <strong>Title:</strong> {result.meta_tags.title}
                    </p>
                    <p>
                      <strong>Description:</strong>{" "}
                      {result.meta_tags.description}
                    </p>
                    <p>
                      <strong>Keywords:</strong>{" "}
                      {result.meta_tags.keywords.join(", ")}
                    </p>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Content Structure</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <h3 className="font-semibold">Headings</h3>
                      <p>H1: {result.headings.h1}</p>
                      <p>H2: {result.headings.h2}</p>
                      <p>H3: {result.headings.h3}</p>
                    </div>
                    <div>
                      <h3 className="font-semibold">Images</h3>
                      <p>Total: {result.images.total}</p>
                      <p>With Alt: {result.images.with_alt}</p>
                      <p>Without Alt: {result.images.without_alt}</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Links and Performance</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <h3 className="font-semibold">Links</h3>
                      <p>Internal: {result.links.internal}</p>
                      <p>External: {result.links.external}</p>
                    </div>
                    <div>
                      <h3 className="font-semibold">Performance</h3>
                      <p>Load Time: {result.performance.load_time}</p>
                      <p>
                        Mobile Responsive:{" "}
                        {result.performance.mobile_responsive ? "Yes" : "No"}
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
