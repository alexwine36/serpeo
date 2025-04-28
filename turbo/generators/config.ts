import type { PlopTypes } from "@turbo/gen";
import path from "node:path";
import { capitalize, pipe, split, toCamelCase, toKebabCase } from "remeda";

type TurboAnswers = {
  turbo: {
    paths: {
      cwd: string;
      root: string;
      workspace: string;
    };
  };
};

export default function generator(plop: PlopTypes.NodePlopAPI): void {
  plop.setGenerator("plugin", {
    description: "Generate a new plugin",
    prompts: [
      {
        type: "input",
        name: "name",
        message:
          "What is the name of the plugin? Ps. don't use the word plugin in the name",
      },
      {
        type: "input",
        name: "description",
        message: "What is the description of the plugin?",
      },
    ],
    actions: (rawData) => {
      const modData = rawData as TurboAnswers & {
        name: string;
        description: string;
      };

      const libPath = path.join(
        modData.turbo.paths.workspace,
        "crates/seo-plugins"
      );

      const pluginModuleName = pipe(
        modData.name,
        toKebabCase,
        split("-"),
        (res) => res.join("_")
      );

      const pluginId = pluginModuleName;

      const pluginFileName = pipe(pluginModuleName, (name) => `${name}.rs`);

      const pluginName = pipe(modData.name, toCamelCase(), capitalize);
      const pluginPath = path.join(libPath, "src/plugins", pluginFileName);
      const modPath = path.join(libPath, "src/plugins/mod.rs");
      const templatePath = path.join(
        modData.turbo.paths.workspace,
        "turbo/generators/templates/plugin.rs.hbs"
      );
      const actions: PlopTypes.Actions = [];
      const data = {
        ...modData,
        modPath,
        libPath,
        pluginModuleName,
        pluginFileName,
        pluginPath,
        pluginId,
        pluginName,
        templatePath,
      };

      actions.push({
        type: "append",
        path: modPath,
        pattern: ";",
        template: `pub mod ${pluginModuleName};`,
      });
      actions.push({
        type: "add",
        path: pluginPath,
        templateFile: templatePath,
      });

      // biome-ignore lint/suspicious/noExplicitAny: <explanation>
      return actions.map((a) => ({ ...(a as any), data }));
    },
  });
}
