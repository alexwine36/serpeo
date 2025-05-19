import { Button } from "@repo/ui/components/button";
import { Link } from "@tanstack/react-router";
import { RefreshCcw, ScanSearch } from "lucide-react";
import type { SiteWithSiteRuns } from "../../../generated/bindings";

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
    <div className="flex items-center justify-between gap-2">
      <h1>{site.site.url}</h1>

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
              to={"/analysis/$siteRunId"}
              params={{ siteRunId: mostRecentRun.id.toString() }}
            >
              <ScanSearch className="h-4 w-4" />
            </Link>
          </Button>
        )}
      </div>
    </div>
  );
};
