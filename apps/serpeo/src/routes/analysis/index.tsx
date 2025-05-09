import { createFileRoute } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { crawlResultAtom } from "../../atoms/crawl-result";
import { IssueCategoryDetail } from "../../components/display/issue-category-detail";
import { IssueCategoryOverview } from "../../components/display/issue-category-overview";
import { LinkDisplay } from "../../components/display/link-display";

export const Route = createFileRoute("/analysis/")({
  component: RouteComponent,
});

function RouteComponent() {
  const [result, setResult] = useAtom(crawlResultAtom);
  if (!result.total_pages) {
    return <div>No results found</div>;
  }
  return (
    <div className="container mt-8 space-y-6">
      <LinkDisplay />
      <IssueCategoryOverview />
      <IssueCategoryDetail />
    </div>
  );
}
