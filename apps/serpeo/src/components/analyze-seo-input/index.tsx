import { UrlInput } from "@repo/ui/components/url-input";
import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { commands, events } from "../../generated/bindings";
import { useSitesQuery } from "../../queries/sites";

export type AnalyzeSeoInputProps = {
  onSubmit?: (url: string) => void;
};

export const AnalyzeSeoInput = (props: AnalyzeSeoInputProps) => {
  const { data: sites } = useSitesQuery();
  const navigate = useNavigate();
  const analyzeSeo = async (url: string) => {
    try {
      // setLoading(true);
      // setResult(RESET);
      const _analysis = await commands.analyzeUrlSeo(url);
    } catch (error) {
      console.error("Error analyzing SEO:", error);
    }
  };

  useEffect(() => {
    events.siteRunIdSet.listen((e) => {
      console.log("siteRunIdSet");
      console.log(e.payload);
      navigate({
        to: "/analysis/$site-run-id",
        params: { "site-run-id": `${e.payload.site_run_id}` },
      });
    });
  }, [navigate]);

  return (
    <UrlInput
      className="md:w-sm lg:w-md"
      onSubmit={(url) => {
        analyzeSeo(url);
      }}
      previousUrls={sites?.map(({ site }) => site.url)}
    />
  );
};
