import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Link } from "@tanstack/react-router";
import { RefreshCcw, ScanSearch } from "lucide-react";
import type { SiteWithSiteRuns } from "../../../generated/bindings";
import { ChartWrapper } from "./overview-chart";
type Props = {
  sites: SiteWithSiteRuns[];
};

export const SitesOverview = ({ sites }: Props) => {
  return (
    <div className="flex flex-col gap-4">
      {sites.map((site) => (
        <SiteDisplay key={site.site.id} site={site} />
      ))}
    </div>
  );
};

const SiteDisplay = ({ site }: { site: SiteWithSiteRuns }) => {
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

          <div className="flex items-center gap-2">
            <Button disabled size="icon" asChild variant="outline">
              <Link
                // disabled
                to={"/"}
                //   params={{ siteId: site.site.id.toString() }}
              >
                <RefreshCcw className="h-4 w-4" />
              </Link>
            </Button>
            {mostRecentRun && (
              <Button size="icon" asChild variant="outline">
                <Link
                  to={"/analysis/$site-run-id"}
                  params={{ "site-run-id": mostRecentRun.id.toString() }}
                >
                  <ScanSearch className="h-4 w-4" />
                </Link>
              </Button>
            )}
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ChartWrapper siteId={site.site.id.toString()} />
      </CardContent>
    </Card>
  );
};
