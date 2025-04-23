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
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
export const Route = createFileRoute("/about")({
  component: About,
});

function About() {
  const [url, setUrl] = useState("https://stem-programs.newspacenexus.org/");
  const [loading, setLoading] = useState(false);

  const crawlSeo = async () => {
    try {
      setLoading(true);
      const crawl = await invoke("crawl_seo", { url });
      console.log("Crawls", crawl);
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
      <Card>
        <CardHeader>
          <CardTitle>Results</CardTitle>
        </CardHeader>
      </Card>
    </div>
  );
}
