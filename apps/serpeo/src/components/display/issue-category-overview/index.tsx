import { CircleProgress } from "@repo/ui/components/circle-progress";
import { useAtomValue } from "jotai";
import {
  type IssueCategoryItem,
  issueCategoriesAtom,
} from "../../../atoms/crawl-result";

export const IssueCategoryOverview = () => {
  const categories = useAtomValue(issueCategoriesAtom);
  return (
    <div className="flex w-full flex-row flex-wrap justify-evenly gap-4">
      {Object.entries(categories).map(([category, issues]) => (
        <IssueCategoryOverviewItem
          key={category}
          category={category}
          issues={issues}
        />
      ))}
    </div>
  );
};

const IssueCategoryOverviewItem = ({
  category,
  issues,
}: { category: string; issues: IssueCategoryItem[] }) => {
  return (
    <div>
      <div className="flex flex-col items-center gap-2">
        <h2 className="font-bold text-lg">{category}</h2>
        <CircleProgress
          size={100}
          showPercentage
          value={issues.filter((i) => i.passed).length}
          maxValue={issues.length}
        />
      </div>
    </div>
  );
};
