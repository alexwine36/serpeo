import { AnimatedNumber } from "@repo/ui/components/animated-number";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { useAtomValue } from "jotai";
import { linksAtom } from "../../../atoms/crawl-result";
import {
  type SiteRunLinkModified,
  useSiteRunLinkCountsQuery,
} from "../../../queries/sites";

export const LinkDisplayOld = () => {
  const links = useAtomValue(linksAtom);

  const displayOpts: (keyof typeof links)[] = ["Total", "Internal", "External"];
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3">
      {displayOpts.map((linkType) => {
        const linkCount = links[linkType]?.length ?? 0;
        return (
          <Card key={linkType}>
            <CardHeader>
              <CardTitle>{linkType}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-end justify-end font-bold text-2xl">
                <AnimatedNumber value={linkCount} />
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
};

type LinkDisplayProps = {
  siteRunId: number;
};

export const LinkDisplay = ({ siteRunId }: LinkDisplayProps) => {
  const { data: siteRunLinkCounts } = useSiteRunLinkCountsQuery(siteRunId);

  const displayOpts: SiteRunLinkModified["db_link_type"][] = [
    "Total",
    "Internal",
    "External",
  ];

  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3">
      {displayOpts.map((linkType) => {
        const linkCount =
          siteRunLinkCounts?.find((count) => count.db_link_type === linkType)
            ?.count ?? 0;
        return (
          <Card key={linkType}>
            <CardHeader>
              <CardTitle>{linkType}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-end justify-end font-bold text-2xl">
                <AnimatedNumber value={linkCount} />
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
};
