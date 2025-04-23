import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";

interface SeoAnalysisResult {
  meta_tags: {
    title: string;
    description: string;
    keywords: string[];
  };
  headings: {
    h1: number;
    h2: number;
    h3: number;
  };
  images: {
    total: number;
    withAlt: number;
    withoutAlt: number;
  };
  links: {
    internal: number;
    external: number;
  };
  performance: {
    load_time: string;
    mobile_responsive: boolean;
  };
}

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [url, setUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<SeoAnalysisResult | null>(null);

  const analyzeSeo = async () => {
    try {
      setLoading(true);
      const analysis = await invoke("analyze_seo", { url });
      console.log("Analysis", analysis);
      setResult(analysis as SeoAnalysisResult);
    } catch (error) {
      console.error("Error analyzing SEO:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-4 max-w-4xl">
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
                      <p>With Alt: {result.images.withAlt}</p>
                      <p>Without Alt: {result.images.withoutAlt}</p>
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
