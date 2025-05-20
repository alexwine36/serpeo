import { createFileRoute, useLoaderData } from "@tanstack/react-router";
import { SitesOverview } from "../../../components/sites/overview";
import { commands } from "../../../generated/bindings";

export const Route = createFileRoute("/analysis/sites/")({
  loader: async () => {
    const sites = await commands.getSites();
    if (sites.status === "error") {
      throw new Error(sites.error);
    }
    return {
      sites: sites.data,
    };
  },
  component: RouteComponent,
});

function RouteComponent() {
  //   console.log(props);
  const { sites } = useLoaderData({ from: "/analysis/sites/" });
  console.log(sites);
  return (
    <div>
      <SitesOverview sites={sites} />
    </div>
  );
}
