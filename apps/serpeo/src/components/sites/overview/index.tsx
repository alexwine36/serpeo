import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Link } from "@tanstack/react-router";
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { RefreshCcw, ScanSearch } from "lucide-react";
import type { SiteWithSiteRuns } from "../../../generated/bindings";
import { useAnalyzeSeoInput } from "../../analyze-seo-input/use-analyze-seo-input";
import { ChartWrapper } from "./overview-chart";
dayjs.extend(relativeTime);
type Props = {
  sites: SiteWithSiteRuns[];
};

export const SitesOverview = ({ sites }: Props) => {
  const { analyzeSeo } = useAnalyzeSeoInput();
  return (
    <div className="flex flex-col gap-4">
      {sites.map((site) => (
        <SiteDisplay
          key={site.site.id}
          site={site}
          onRefresh={() => analyzeSeo(site.site.url)}
        />
      ))}
    </div>
  );
};

const SiteDisplay = ({
  site,
  onRefresh,
}: {
  site: SiteWithSiteRuns;
  onRefresh?: () => void;
}) => {
  const lastRun = dayjs(site.last_site_run_at);
  const siteRuns = site.site_runs.sort(
    (a, b) =>
      new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  console.log("SITE RUNS", siteRuns);

  const mostRecentRun = siteRuns[0];
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between gap-2">
          {site.site.name || site.site.url}

          <div className="flex items-center gap-1">
            {onRefresh && (
              <Button size="icon" variant="outline" onClick={onRefresh}>
                <RefreshCcw className="h-4 w-4" />
              </Button>
            )}

            {mostRecentRun && (
              <Button size="icon" asChild variant="outline">
                <Link
                  to={"/analysis/$site-run-id"}
                  params={{ "site-run-id": mostRecentRun.id.toString() }}
                  viewTransition={{
                    types: ["slide-left"],
                  }}
                >
                  <ScanSearch className="h-4 w-4" />
                </Link>
              </Button>
            )}
          </div>
        </CardTitle>
        <CardDescription>{lastRun.fromNow()}</CardDescription>
      </CardHeader>
      <CardContent>
        <ChartWrapper siteId={site.site.id.toString()} />
      </CardContent>
    </Card>
  );
};
