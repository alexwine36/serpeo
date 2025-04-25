import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Progress } from "@repo/ui/components/progress";
import { cn } from "@repo/ui/lib/utils";
import { Check, Loader, LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
import {
  commands,
  events,
  type AnalysisProgress,
  type AnalysisResult,
  type CrawlResult,
} from "../../generated/bindings";

type AnalysisProgressDisplayProps = {
  results: CrawlResult;
  url: string;
};
export const AnalysisProgressDisplay = ({
  results,
  url,
}: AnalysisProgressDisplayProps) => {
  const [res, setRes] = useState<AnalysisProgress | undefined>();
  // const [analysisResult, setAnalysisResult] = useState<>([])
  useEffect(() => {
    commands.analyzeCrawlSeo(url, results, false);
  }, [url, results]);

  useEffect(() => {
    events.analysisProgress.listen((e) => {
      console.log("PROGRESS", e);
      setRes(e.payload);
      //   setRes(JSON.stringify(e.payload, null, 3));
    });
    // listen("analysis-progress", (e) => {
    //   console.log("PROGRESS", e);
    //   setRes(JSON.stringify(e.payload, null, 3));
    // });
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Results</CardTitle>
        <CardDescription>Page Count: {results.total_pages}</CardDescription>
      </CardHeader>
      <CardContent>
        {res && (
          <Progress value={(res.completed_urls / res.total_urls) * 100} />
        )}
        {res?.completed_urls} / {res?.total_urls}
        <div>
          {res &&
            Object.entries(res?.results).map(([key, val]) => {
              return (
                <div className="flex gap-2" key={key}>
                  <span className="s-8">
                    <StatusDisplay status={val?.status || "Pending"} />
                  </span>
                  <div className="grid grid-cols-4">
                    <p className="truncate">{val?.analysis.base_info.path}</p>
                    <p>{val?.analysis.meta_tags.title}</p>
                  </div>
                </div>
              );
            })}
        </div>
      </CardContent>
    </Card>
  );
};

const StatusDisplay = ({ status }: { status: AnalysisResult["status"] }) => {
  const baseClass = "s-10";
  if (status === "Complete") {
    return <Check className={cn(baseClass)} />;
  }
  if (status === "InProgress") {
    return <LoaderCircle className={cn(baseClass)} />;
  }
  return <Loader className={cn(baseClass)} />;
};
