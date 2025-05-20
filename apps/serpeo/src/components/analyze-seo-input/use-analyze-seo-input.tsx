import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { events, commands } from "../../generated/bindings";

export const useAnalyzeSeoInput = () => {
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

  return {
    analyzeSeo,
  };
};
