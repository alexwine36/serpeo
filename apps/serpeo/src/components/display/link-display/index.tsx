import { AnimatedNumber } from "@repo/ui/components/animated-number";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { useAtomValue } from "jotai";
import { linksAtom } from "../../../atoms/crawl-result";

export const LinkDisplay = () => {
  const links = useAtomValue(linksAtom);

  const displayOpts: (keyof typeof links)[] = ["Total", "Internal", "External"];
  return (
    <div className="flex gap-4">
      {displayOpts.map((linkType) => {
        const linkCount = links[linkType]?.length ?? 0;
        return (
          <Card className="w-full" key={linkType}>
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
