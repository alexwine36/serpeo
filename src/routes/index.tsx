import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }
  return (
    <main className="container mx-auto">
      <form
        className="flex flex-row gap-4"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <Input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        {/* <button type="submit">Greet</button> */}
        <Button type="submit">Greet</Button>
      </form>
      <p>{greetMsg}</p>
      <div className="p-2">
        <h3>Welcome Home!</h3>
      </div>
    </main>
  );
}
