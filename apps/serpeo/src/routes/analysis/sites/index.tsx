import { SwirlingEffectSpinner } from "@repo/ui/components/customized/spinner/swirling-effect-spinner";
import { createFileRoute } from "@tanstack/react-router";
import { SitesOverview } from "../../../components/sites/overview";
import { useSitesQuery } from "../../../queries/sites";

export const Route = createFileRoute("/analysis/sites/")({
  // loader: async () => {
  //   const sites = await commands.getSites();
  //   if (sites.status === "error") {
  //     throw new Error(sites.error);
  //   }
  //   return {
  //     sites: sites.data,
  //   };
  // },
  component: RouteComponent,
});

function RouteComponent() {
  //   console.log(props);
  const { data: sites } = useSitesQuery();
  if (!sites) {
    return (
      <div className="flex h-full w-full items-center justify-center">
        <SwirlingEffectSpinner />
      </div>
    );
  }
  console.log(sites);
  return (
    <div>
      <SitesOverview sites={sites} />
    </div>
  );
}
