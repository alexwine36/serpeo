import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { createFileRoute, useLoaderData } from "@tanstack/react-router";
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { commands } from "../../generated/bindings";

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
  return (
    <Card>
      <CardHeader>
        <CardTitle>Site Run</CardTitle>
        <CardDescription>{siteRun.created_at.fromNow()}</CardDescription>
      </CardHeader>
    </Card>
  );
}
