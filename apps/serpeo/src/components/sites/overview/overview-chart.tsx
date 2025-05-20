"use client";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@repo/ui/components/chart";
import { SwirlingEffectSpinner } from "@repo/ui/components/customized/spinner/swirling-effect-spinner";
import dayjs from "dayjs";
import { CartesianGrid, Line, LineChart, XAxis } from "recharts";
import type { RuleCategory } from "../../../generated/bindings";
import { useSiteCategoryHistoryQuery } from "../../../queries/sites";

const chartConfig: Record<RuleCategory, { label: string; color: string }> = {
  Accessibility: {
    label: "Accessibility",
    color: "var(--chart-1)",
  },
  BestPractices: {
    label: "Best Practices",
    color: "var(--chart-2)",
  },
  Performance: {
    label: "Performance",
    color: "var(--chart-3)",
  },
  SEO: {
    label: "SEO",
    color: "var(--chart-4)",
  },
} satisfies Record<RuleCategory, { label: string; color: string }>;

type ChartWrapperProps = {
  siteId: string;
};

type ChartData = {
  [key in RuleCategory]?: number;
} & {
  created_at: string;
};

export const ChartWrapper = ({ siteId }: ChartWrapperProps) => {
  const { data, isLoading } = useSiteCategoryHistoryQuery(Number(siteId));

  const chartData: ChartData[] | undefined = data?.map((item) => {
    const data: ChartData = {
      created_at: dayjs(item.created_at).format("M/D"),
    };
    // biome-ignore lint/nursery/useGuardForIn: <explanation>
    for (const key in item.data) {
      const cat = item.data[key as RuleCategory];
      if (cat) {
        data[key as RuleCategory] = (cat.passed / cat.total) * 100;
      }
    }
    return data;
  });

  if (isLoading) {
    return (
      <div className="aspect-3/1">
        <div className="flex h-full w-full items-center justify-center">
          <SwirlingEffectSpinner size="lg" />
        </div>
      </div>
    );
  }
  if (!chartData) {
    return <div>No data</div>;
  }
  return <OverviewChart chartData={chartData} />;
};

export function OverviewChart({ chartData }: { chartData: ChartData[] }) {
  return (
    <ChartContainer className="aspect-3/1" config={chartConfig}>
      <LineChart
        accessibilityLayer
        data={chartData}
        margin={{
          left: 12,
          right: 12,
        }}
      >
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="created_at"
          tickLine={false}
          axisLine={false}
          tickMargin={8}
          //   tickFormatter={(value) => value.slice(0, 3)}
        />
        <ChartTooltip cursor={false} content={<ChartTooltipContent />} />

        {Object.entries(chartConfig).map(([key, _config]) => (
          <Line
            key={key}
            dataKey={key}
            type="monotone"
            stroke={`var(--color-${key})`}
            strokeWidth={2}
            dot={false}
          />
        ))}
      </LineChart>
    </ChartContainer>
  );
}
