import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@repo/ui/components/dialog";
import { Separator } from "@repo/ui/components/separator";
import { useQueryClient } from "@tanstack/react-query";
import {
  createFileRoute,
  useLoaderData,
  useRouter,
} from "@tanstack/react-router";
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { useEffect, useState } from "react";
import { AnalysisStatus } from "../../components/analysis-status";
import { IssueCategoryDetail } from "../../components/display/issue-category-detail";
import { IssueCategoryOverview } from "../../components/display/issue-category-overview";
import { LinkDisplay } from "../../components/display/link-display";
import { commands, events } from "../../generated/bindings";

dayjs.extend(relativeTime);
export const Route = createFileRoute("/analysis/$site-run-id")({
  loader: async ({ params }) => {
    const siteRunId = params["site-run-id"];
    console.log(siteRunId);
    const siteRun = await commands.getSiteRunById(Number(siteRunId));

    if (siteRun.status === "error") {
      throw new Error(siteRun.error);
    }
    const site = await commands.getSiteById(siteRun.data.site_id);
    if (site.status === "error") {
      throw new Error(site.error);
    }
    return {
      siteRun: { ...siteRun.data, created_at: dayjs(siteRun.data.created_at) },
      site: site.data,
    };
  },
  component: SiteRun,
});

function SiteRun() {
  const { siteRun, site } = useLoaderData({ from: "/analysis/$site-run-id" });
  const queryClient = useQueryClient();

  const router = useRouter();

  const [done, setDone] = useState(true);
  useEffect(() => {
    const doneDelay = () => {
      setTimeout(() => {
        setDone(false);
      }, 250);
    };

    if (siteRun.status !== "Finished") {
      doneDelay();
    }

    const unsubscribe = events.analysisFinished.listen((e) => {
      const siteRunId = e.payload.site_run_id;
      if (siteRunId === siteRun.id) {
        setDone(true);
        router.invalidate();
        queryClient.invalidateQueries({
          predicate: (query) =>
            query.queryKey[0] === "siteRun" || query.queryKey[0] === "sites",
        });
      }
    });

    return () => {
      unsubscribe.then((unsubscribe) => {
        unsubscribe();
      });
    };
  }, [siteRun.id, router, queryClient, siteRun.status]);

  console.log(siteRun);
  return (
    <Card>
      <CardHeader>
        <CardTitle>
          Results for{" "}
          {site && (
            <span className="text-muted-foreground text-sm">{site.url}</span>
          )}
        </CardTitle>
        <CardDescription>{siteRun.created_at.fromNow()}</CardDescription>
      </CardHeader>
      <CardContent>
        {!done && (
          <Dialog open={!done}>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Run Status</DialogTitle>
              </DialogHeader>
              <div>
                <AnalysisStatus />
              </div>
            </DialogContent>
          </Dialog>
        )}

        <div className="flex flex-col gap-4">
          <Separator />
          <LinkDisplay siteRunId={siteRun.id} />
          <IssueCategoryOverview siteRunId={siteRun.id} />
          <IssueCategoryDetail siteRunId={siteRun.id} />
        </div>
      </CardContent>
    </Card>
  );
}
