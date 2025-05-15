import { useQuery } from "@tanstack/react-query";
import { commands } from "../generated/bindings";

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
