import { UrlInput } from "@repo/ui/components/url-input";
import { useNavigate } from "@tanstack/react-router";
import { useSitesQuery } from "../../queries/sites";
import { useAnalyzeSeoInput } from "./use-analyze-seo-input";

export type AnalyzeSeoInputProps = {
  onSubmit?: (url: string) => void;
};

export const AnalyzeSeoInput = (props: AnalyzeSeoInputProps) => {
  const { data: sites } = useSitesQuery();
  const navigate = useNavigate();

  const { analyzeSeo } = useAnalyzeSeoInput();

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
