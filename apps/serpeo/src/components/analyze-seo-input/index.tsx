import { UrlInput } from "@repo/ui/components/url-input";
import { useSitesQuery } from "../../queries/sites";
import { useAnalyzeSeoInput } from "./use-analyze-seo-input";

export type AnalyzeSeoInputProps = {
  onSubmit?: (url: string) => void;
};

export const AnalyzeSeoInput = (_props: AnalyzeSeoInputProps) => {
  const { data: sites } = useSitesQuery();

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
