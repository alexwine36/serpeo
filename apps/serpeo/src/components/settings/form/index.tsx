import { zodResolver } from "@hookform/resolvers/zod";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Form } from "@repo/ui/components/form";
import { FormInput } from "@repo/ui/custom/form-input";
import { useEffect } from "react";
import { type SubmitHandler, useForm, useWatch } from "react-hook-form";
import { z } from "zod";
import type { CrawlSettingsStore } from "../../../generated/bindings";
const schema: z.ZodSchema<CrawlSettingsStore> = z.object({
  max_concurrent_requests: z.coerce.number().min(1), //.max(20),
  request_delay_ms: z.coerce.number().min(0).max(1000),
});

type SettingsFormProps = {
  config: CrawlSettingsStore;
  setConfig: (config: CrawlSettingsStore) => void;
};

export const SettingsForm = ({ config, setConfig }: SettingsFormProps) => {
  const form = useForm<CrawlSettingsStore>({
    resolver: zodResolver(schema),
    defaultValues: config,
  });

  const watchedValues = useWatch({ control: form.control });

  const onSubmit: SubmitHandler<CrawlSettingsStore> = (data) => {
    setConfig(data);
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (!form.formState.isDirty) {
      return;
    }

    form.handleSubmit(onSubmit)();
  }, [watchedValues, form.handleSubmit, form.formState.isDirty]);

  return (
    <Form {...form}>
      <form className="space-y-2" onSubmit={form.handleSubmit(onSubmit)}>
        <Card>
          <CardHeader>
            <CardTitle>Requests</CardTitle>
          </CardHeader>
          <CardContent className="grid grid-cols-2 gap-4">
            <FormInput
              control={form.control}
              type="number"
              name="max_concurrent_requests"
              label="Max Concurrent Requests"
              description="The maximum number of concurrent requests"
            />

            <FormInput
              control={form.control}
              type="number"
              name="request_delay_ms"
              label="Request Delay (ms)"
              description="The delay between requests"
            />
          </CardContent>
        </Card>
        {/* <div className="mt-4 flex justify-end">
          <Button type="submit" disabled={isLoading}>
            {isLoading ? "Saving..." : "Save Settings"}
          </Button>
        </div> */}
      </form>
    </Form>
  );
};
