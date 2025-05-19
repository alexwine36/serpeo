import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Separator } from "@repo/ui/components/separator";
import { createFileRoute, useLoaderData } from "@tanstack/react-router";
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { IssueCategoryOverview } from "../../components/display/issue-category-overview";
import { LinkDisplay } from "../../components/display/link-display";
import { commands } from "../../generated/bindings";
import { useSiteByIdQuery } from "../../queries/sites";

dayjs.extend(relativeTime);
export const Route = createFileRoute("/analysis/$siteRunId")({
  loader: async ({ params }) => {
    const siteRunId = params.siteRunId;
    console.log(siteRunId);
    const siteRun = await commands.getSiteRunById(Number(siteRunId));
    if (siteRun.status === "error") {
      throw new Error(siteRun.error);
    }
    return {
      siteRun: { ...siteRun.data, created_at: dayjs(siteRun.data.created_at) },
    };
  },
  component: SiteRun,
});

function SiteRun() {
  const { siteRun } = useLoaderData({ from: "/analysis/$siteRunId" });
  console.log(siteRun);
  const { data: site } = useSiteByIdQuery(siteRun.site_id);
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
        <div className="flex flex-col gap-4">
          <Separator />
          <LinkDisplay siteRunId={siteRun.id} />
          <IssueCategoryOverview siteRunId={siteRun.id} />
        </div>
      </CardContent>
    </Card>
  );
}
