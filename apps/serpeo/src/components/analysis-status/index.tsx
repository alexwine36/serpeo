import { CircleProgress } from "@repo/ui/components/circle-progress";
import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { crawlResultAtom } from "../../atoms/crawl-result";
import { events } from "../../generated/bindings";

export const AnalysisStatus = () => {
  const [progress, setProgress] = useState<
    | {
        value: number;
        maxValue: number;
        url: string;
      }
    | undefined
  >(undefined);
  const crawlResult = useAtomValue(crawlResultAtom);

  useEffect(() => {
    events.analysisProgress.listen((e) => {
      //   console.log(e.payload);
      setProgress({
        value: e.payload.completed_pages,
        maxValue: e.payload.total_pages,
        url: e.payload.url || "",
      });
    });
  }, []);

  if (!progress || crawlResult.total_pages !== 0) {
    return null;
  }

  return (
    <div className="mt-8 flex flex-row items-center justify-center gap-4">
      <div className="flex flex-col items-center justify-center gap-2">
        <CircleProgress
          strokeWidth={10}
          size={150}
          value={progress.value}
          maxValue={progress.maxValue}
        />

        <p className="text-muted-foreground text-sm">{progress.url}</p>
      </div>
    </div>
  );
};
