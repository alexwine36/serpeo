import { Badge } from "@repo/ui/components/badge";
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
import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { Config } from "../../../generated/bindings";
import { commands } from "../../../generated/bindings";
import { SettingsForm } from "../form";

type SettingsCardProps = {
  //   onSubmit?: (config: Config) => void;
  collapsible?: boolean;
};

export const SettingsCard = ({ collapsible = false }: SettingsCardProps) => {
  const [config, setConfig] = useState<Config | undefined>();
  const [isLoading, setIsLoading] = useState(false);

  const [isOpen, setIsOpen] = useState(!collapsible);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const result = await commands.getConfig();
      if (result.status === "ok") {
        console.log("Loaded config", result.data);
        setConfig(result.data);
      } else {
        toast.error("Failed to load config");
      }
    } catch (error) {
      toast.error("Failed to load config");
    }
  };

  const handleSave = async (data: Config) => {
    setIsLoading(true);
    try {
      const result = await commands.setConfig(data);

      if (result.status === "ok") {
        toast.success("Settings saved successfully");
        await loadConfig();
        // onSubmit?.(data);
      } else {
        toast.error("Failed to save settings");
      }
    } catch (error) {
      toast.error("Failed to save settings");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Collapsible
      open={isOpen}
      onOpenChange={setIsOpen}

      // className="w-[350px] space-y-2"
    >
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
            <br />
            {config?.base_url && (
              <Badge variant="outline">{config?.base_url}</Badge>
            )}
          </CardDescription>
        </CardHeader>
        <CollapsibleContent>
          <CardContent className="mt-4 space-y-4">
            <Separator />
            {config && (
              <SettingsForm
                config={config}
                setConfig={handleSave}
                isLoading={isLoading}
              />
            )}
          </CardContent>
        </CollapsibleContent>
      </Card>
    </Collapsible>
  );
};
