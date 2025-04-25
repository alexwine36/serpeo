import { Button } from "@repo/ui/components/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@repo/ui/components/card";
import { Input } from "@repo/ui/components/input";
import { Label } from "@repo/ui/components/label";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import type { Config } from "../generated/bindings";
import { commands } from "../generated/bindings";

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});

function SettingsPage() {
  const [config, setConfig] = useState<Config>({
    base_url: "",
    max_concurrent_requests: 5,
    request_delay_ms: 100,
  });
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const result = await commands.getConfig();
      if (result.status === "ok") {
        setConfig(result.data);
      } else {
        toast.error("Failed to load config");
      }
    } catch (error) {
      toast.error("Failed to load config");
    }
  };

  const handleSave = async () => {
    setIsLoading(true);
    try {
      const result = await commands.setConfig(config);
      if (result.status === "ok") {
        toast.success("Settings saved successfully");
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
    <div className="container mx-auto py-8">
      <Card>
        <CardHeader>
          <CardTitle>Settings</CardTitle>
          <CardDescription>
            Configure your SEO analysis settings
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="base_url">Base URL</Label>
            <Input
              id="base_url"
              value={config.base_url}
              onChange={(e) =>
                setConfig({ ...config, base_url: e.target.value })
              }
              placeholder="https://example.com"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="max_concurrent_requests">
              Max Concurrent Requests
            </Label>
            <Input
              id="max_concurrent_requests"
              type="number"
              value={config.max_concurrent_requests}
              onChange={(e) =>
                setConfig({
                  ...config,
                  max_concurrent_requests: Number.parseInt(e.target.value) || 5,
                })
              }
              min="1"
              max="10"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="request_delay_ms">Request Delay (ms)</Label>
            <Input
              id="request_delay_ms"
              type="number"
              value={config.request_delay_ms}
              onChange={(e) =>
                setConfig({
                  ...config,
                  request_delay_ms: Number.parseInt(e.target.value) || 100,
                })
              }
              min="0"
              max="1000"
            />
          </div>

          <Button onClick={handleSave} disabled={isLoading}>
            {isLoading ? "Saving..." : "Save Settings"}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
