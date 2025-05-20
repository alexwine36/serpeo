import { useQuery } from "@tanstack/react-query";
import { type DbLinkType, commands } from "../generated/bindings";

export const useSitesQuery = () => {
  return useQuery({
    queryKey: ["sites", "all"],
    queryFn: async () => {
      const sites = await commands.getSites();
      if (sites.status === "ok") {
        return sites.data;
      }
      return [];
    },
  });
};

export const useSiteByIdQuery = (id: number) => {
  return useQuery({
    queryKey: ["site", id],
    queryFn: async () => {
      const site = await commands.getSiteById(id);
      if (site.status === "ok") {
        return site.data;
      }
      return null;
    },
  });
};

export const useSiteRunCategoryResultQuery = (siteRunId: number) => {
  return useQuery({
    queryKey: ["siteRunCategoryResult", siteRunId],
    queryFn: async () => {
      const siteRunCategoryResult = await commands.getCategoryResult(siteRunId);
      if (siteRunCategoryResult.status === "ok") {
        return siteRunCategoryResult.data;
      }
      return null;
    },
  });
};

export type SiteRunLinkModified = {
  db_link_type: DbLinkType | "Total";
  count: number;
};

export const useSiteRunLinkCountsQuery = (siteRunId: number) => {
  return useQuery({
    queryKey: ["siteRunLinkCounts", siteRunId],
    queryFn: async (): Promise<SiteRunLinkModified[]> => {
      const siteRunLinkCounts = await commands.getSiteRunLinkCounts(siteRunId);
      if (siteRunLinkCounts.status === "ok") {
        const total = siteRunLinkCounts.data.reduce(
          (acc, curr) => acc + curr.count,
          0
        );
        return [
          {
            db_link_type: "Total",
            count: total,
          },
          ...siteRunLinkCounts.data,
        ];
      }
      return [];
    },
  });
};

export const useSiteRunCategoryResultDetailQuery = (
  siteRunId: number,
  passed: boolean | null
) => {
  return useQuery({
    queryKey: ["siteRunCategoryResultDetail", siteRunId, passed],
    queryFn: async () => {
      const siteRunCategoryResultDetail =
        await commands.getCategoryResultDetail(siteRunId, passed);
      if (siteRunCategoryResultDetail.status === "ok") {
        return siteRunCategoryResultDetail.data.data;
      }
      return null;
    },
  });
};
