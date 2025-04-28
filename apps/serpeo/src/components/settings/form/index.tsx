import { zodResolver } from "@hookform/resolvers/zod";
import { Button } from "@repo/ui/components/button";
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
import { isDeepEqual } from "remeda";
import { z } from "zod";
import type { Config } from "../../../generated/bindings";
const schema: z.ZodSchema<Config> = z.object({
  base_url: z.string().url(),
  max_concurrent_requests: z.coerce.number().min(1), //.max(20),
  request_delay_ms: z.coerce.number().min(0).max(1000),
});

type SettingsFormProps = {
  config: Config;
  setConfig: (config: Config) => void;
  isLoading: boolean;
};

export const SettingsForm = ({
  config,
  setConfig,
  isLoading,
}: SettingsFormProps) => {
  const form = useForm<Config>({
    resolver: zodResolver(schema),
    defaultValues: config,
  });

  // // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  // useEffect(() => {
  //   form.reset(config);
  // }, [config]);
  const watchedValues = useWatch({ control: form.control });

  const onSubmit: SubmitHandler<Config> = (data) => {
    if (isDeepEqual(data, config)) {
      return;
    }
    console.log("Submitting", data);
    setConfig(data);
  };

  // const handleSubmit = (data: Config) => {
  //   setConfig(data);
  // };

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (!form.formState.isDirty) {
      return;
    }
    console.log("Watched values", watchedValues);
    form.handleSubmit(onSubmit)();
  }, [watchedValues, form.handleSubmit, form.formState.isDirty]);

  return (
    <Form {...form}>
      <form className="space-y-2" onSubmit={form.handleSubmit(onSubmit)}>
        <FormInput
          control={form.control}
          name="base_url"
          label="Base URL"
          description="The base URL of the API"
        />

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
        <div className="mt-4 flex justify-end">
          <Button type="submit" disabled={isLoading}>
            {isLoading ? "Saving..." : "Save Settings"}
          </Button>
        </div>
      </form>
    </Form>
  );
};
