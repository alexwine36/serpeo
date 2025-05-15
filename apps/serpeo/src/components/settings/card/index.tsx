import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@repo/ui/components/collapsible";
import { Separator } from "@repo/ui/components/separator";
import { cn } from "@repo/ui/lib/utils";
import { ChevronDownIcon } from "lucide-react";
import { useState } from "react";
import { crawlSettingsStore } from "../../../store";
import { SettingsForm } from "../form";

type SettingsCardProps = {
  //   onSubmit?: (config: Config) => void;
  collapsible?: boolean;
};

export const SettingsCard = ({ collapsible = false }: SettingsCardProps) => {
  // const { settings: config, setSettings: setConfig } = useSettings();
  const { data: config, isLoading } = crawlSettingsStore.useQuery();
  const handleSave = crawlSettingsStore.set;
  // const [isLoading, setIsLoading] = useState(false);

  const [isOpen, setIsOpen] = useState(!collapsible);

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      <Card>
        <CardHeader>
          {collapsible ? (
            <CollapsibleTrigger disabled={!collapsible} asChild>
              <div className="flex w-full items-center justify-between gap-2">
                <CardTitle>Settings</CardTitle>
                <Button variant="ghost" size="icon">
                  <ChevronDownIcon
                    className={cn("h-4 w-4 transition-transform", {
                      "rotate-180": isOpen,
                    })}
                  />
                </Button>
              </div>
            </CollapsibleTrigger>
          ) : (
            <CardTitle>Settings</CardTitle>
          )}

          <CardDescription>
            Configure your SEO analysis settings
            {/* <br /> */}
            {/* {config?.base_url && (
              <Badge variant="outline">{config?.base_url}</Badge>
            )} */}
          </CardDescription>
        </CardHeader>
        <CollapsibleContent>
          <CardContent className="mt-4 space-y-4">
            <Separator />
            {config && <SettingsForm config={config} setConfig={handleSave} />}
          </CardContent>
        </CollapsibleContent>
      </Card>
    </Collapsible>
  );
};
