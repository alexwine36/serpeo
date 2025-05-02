import { Badge, type BadgeProps } from "@repo/ui/components/badge";
import type { IssueCategoryItem } from "../../atoms/crawl-result";

export const SeverityBadge = ({
  severity,
}: { severity: IssueCategoryItem["severity"] }) => {
  const outlineColor: Record<
    IssueCategoryItem["severity"],
    BadgeProps["outlineColor"]
  > = {
    Error: "red",
    Warning: "orange",
    Info: "blue",
    Critical: "red",
  };
  return (
    <Badge variant="outline" outlineColor={outlineColor[severity]}>
      {severity}
    </Badge>
  );
};
