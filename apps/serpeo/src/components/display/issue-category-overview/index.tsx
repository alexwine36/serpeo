import { CircleProgress } from "@repo/ui/components/circle-progress";
import { useAtomValue } from "jotai";
import { issueCategoriesAtom } from "../../../atoms/crawl-result";
import { useSiteRunCategoryResultQuery } from "../../../queries/sites";
// DEPRECATED
export const IssueCategoryOverviewOld = () => {
  const categories = useAtomValue(issueCategoriesAtom);
  return (
    <div className="flex w-full flex-row flex-wrap justify-evenly gap-4">
      {Object.entries(categories).map(([category, issues]) => (
        <IssueCategoryOverviewItem
          key={category}
          category={category}
          total={issues.length}
          passed={issues.filter((i) => i.passed).length}
        />
      ))}
    </div>
  );
};

type Props = {
  siteRunId: number;
};

export const IssueCategoryOverview = ({ siteRunId }: Props) => {
  const { data: siteRunCategoryResult } =
    useSiteRunCategoryResultQuery(siteRunId);
  if (!siteRunCategoryResult) {
    return <div>No results found</div>;
  }
  return (
    <div className="flex w-full flex-row flex-wrap justify-evenly gap-4">
      {Object.entries(siteRunCategoryResult.data).map(([category, result]) => (
        <IssueCategoryOverviewItem
          key={category}
          category={category}
          total={result.total}
          passed={result.passed}
        />
      ))}
    </div>
  );
};

const IssueCategoryOverviewItem = ({
  category,
  total,
  passed,
}: { category: string; total: number; passed: number }) => {
  return (
    <div>
      <div className="flex flex-col items-center gap-2">
        <h2 className="font-bold text-lg">{category}</h2>
        <CircleProgress
          size={100}
          showPercentage
          value={passed}
          maxValue={total}
        />
      </div>
    </div>
  );
};
