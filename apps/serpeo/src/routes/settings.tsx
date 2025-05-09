import { createFileRoute } from "@tanstack/react-router";
import { SettingsCard } from "../components/settings/card";

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});

function SettingsPage() {
  return (
    <div className="container mx-auto py-8 [view-transition-name:warp]">
      <SettingsCard />
    </div>
  );
}
