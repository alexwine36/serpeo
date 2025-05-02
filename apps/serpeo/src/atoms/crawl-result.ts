import { atom } from "jotai";
import { focusAtom } from "jotai-optics";
import { atomWithReset } from "jotai/utils";
import { groupBy, prop } from "remeda";
import type { CrawlResult } from "../generated/bindings";

export const crawlResultAtom = atomWithReset<CrawlResult>({
  total_pages: 0,
  page_results: [],
  site_result: [],
});

export const pageResultsAtom = focusAtom(crawlResultAtom, (optic) =>
  optic.prop("page_results")
);
export const siteResultsAtom = focusAtom(crawlResultAtom, (optic) =>
  optic.prop("site_result")
);
export const totalPagesAtom = focusAtom(crawlResultAtom, (optic) =>
  optic.prop("total_pages")
);

export const linksAtom = atom((get) => {
  const pageResults = get(pageResultsAtom);
  return {
    Total: pageResults,
    ...groupBy(pageResults, prop("link_type")),
  };
});

const siteResultsParsed = atom((get) => {
  const siteResults = get(siteResultsAtom);
  return siteResults.flatMap((result) => {
    let urls: string[] = [];
    if (typeof result.context !== "string") {
      if ("Urls" in result.context) {
        urls = result.context.Urls;
      } else if ("Values" in result.context) {
        urls = Object.values(result.context.Values).flat().filter(notNullish);
      }
    }

    return urls.map((url) => ({
      ...result,
      page_url: url,
    }));
  });
});

export const issuesAtom = atom((get) => {
  const pageResults = get(pageResultsAtom);
  const siteResults = get(siteResultsParsed);
  const pageIssues = pageResults
    .flatMap((page) =>
      page.result?.results.filter(notNullish).map((r) => ({
        ...r,
        page_url: page.url,
      }))
    )
    .filter(notNullish);

  return [...pageIssues, ...siteResults].map((issue) => ({
    ...issue,
    pathname: new URL(issue.page_url).pathname,
  }));
});

export const issueCategoriesAtom = atom((get) => {
  const issues = get(issuesAtom);
  return groupBy(issues, prop("category"));
});

export type IssueCategoryItem = Required<
  NonNullable<CrawlResult["page_results"][number]["result"]>
>["results"][number] & {
  page_url: string;
  pathname: string;
};

function notNullish<TValue>(value: TValue | undefined | null): value is TValue {
  return value !== null && value !== undefined; // Can also be `!!value`.
}
