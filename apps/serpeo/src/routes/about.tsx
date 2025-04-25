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
import { useState } from "react";
import { AnalysisProgressDisplay } from "../components/analysis-progress";
import { type CrawlResult, commands } from "../generated/bindings";
export const Route = createFileRoute("/about")({
  component: About,
});

function About() {
  const [url, setUrl] = useState("https://stem-programs.newspacenexus.org/");
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<CrawlResult | null>(null);

  const crawlSeo = async () => {
    try {
      setLoading(true);
      const crawl = await commands.crawlSeo(url);
      console.log("Crawls", crawl);
      if (crawl.status === "ok") {
        setResults(crawl.data);
      }
    } catch (error) {
      console.error("Error analyzing SEO:", error);
    } finally {
      setLoading(false);
    }
  };
  return (
    <div className="flex flex-col gap-4">
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
            <Button onClick={crawlSeo} disabled={loading || !url}>
              {loading ? "Crawling..." : "Crawl"}
            </Button>
          </div>
        </CardContent>
      </Card>
      {results && <AnalysisProgressDisplay url={url} results={results} />}
    </div>
  );
}
