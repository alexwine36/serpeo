import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@repo/ui/components/accordion";
import { Badge } from "@repo/ui/components/badge";
import { useAtomValue } from "jotai";
import { groupBy, prop } from "remeda";
import { issueCategoriesAtom } from "../../../atoms/crawl-result";
import type { FlatRuleResult } from "../../../generated/bindings";
import { useSiteRunCategoryResultDetailQuery } from "../../../queries/sites";
import { SeverityBadge } from "../severity-badge";
export const IssueCategoryDetailOld = () => {
  const categories = useAtomValue(issueCategoriesAtom);
  return (
    <div>
      {Object.entries(categories).map(([category, issues]) => (
        <IssueCategoryDetailItem
          key={category}
          category={category}
          issues={issues}
        />
      ))}
    </div>
  );
};

type IssueCategoryDetailProps = {
  siteRunId: number;
};

export const IssueCategoryDetail = ({
  siteRunId,
}: IssueCategoryDetailProps) => {
  const { data: categories, isLoading } = useSiteRunCategoryResultDetailQuery(
    siteRunId,
    null
  );
  if (isLoading) {
    return <div>Loading...</div>;
  }
  if (!categories) {
    return <div>No data</div>;
  }
  return (
    <div>
      {Object.entries(categories).map(([category, issues]) => (
        <IssueCategoryDetailItem
          key={category}
          category={category}
          issues={issues}
        />
      ))}
    </div>
  );
};

const IssueCategoryDetailItem = ({
  category,
  issues,
}: { category: string; issues: FlatRuleResult[] }) => {
  const failedTests = issues.filter((i) => !i.passed);

  if (failedTests.length === 0) {
    return null;
  }

  const failedTestsByRuleId = Object.entries(
    groupBy(failedTests, prop("name"))
  ).map(([name, tests]) => ({
    name,
    rule_id: tests[0].rule_id,
    tests,
  }));
  //   console.log(failedTestsByRuleId);
  return (
    <div>
      <h2 className="font-bold text-lg">{category}</h2>
      <Accordion type="single" collapsible>
        {failedTestsByRuleId.map((i) => (
          <AccordionItem key={i.rule_id} value={i.rule_id}>
            <AccordionTrigger>
              <div className="grid grid-cols-[45px_80px_1fr] items-center gap-2">
                <Badge variant="outline">{i.tests.length}</Badge>
                <SeverityBadge severity={i.tests[0].severity} />
                {i.name}
              </div>
            </AccordionTrigger>
            <AccordionContent>
              {i.tests.map((t, idx) => {
                return (
                  <div key={`${t.rule_id}-${idx}`}>
                    <p>{t.message}</p>
                    <p>{t.page_url}</p>
                  </div>
                );
              })}
            </AccordionContent>
          </AccordionItem>
        ))}
      </Accordion>
    </div>
  );
};
