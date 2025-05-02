import { atom, useAtom } from "jotai";
import { withAtomEffect } from "jotai-effect";
import { focusAtom } from "jotai-optics";
import { useEffect } from "react";
import { toast } from "sonner";
import { type CrawlConfig, commands } from "../generated/bindings";
// const baseSettingsAtom = atom<CrawlConfig | null>(null);

const settingsAtom = atom<CrawlConfig>({
  base_url: "",
  max_concurrent_requests: 10,
  request_delay_ms: 1000,
});

export const baseUrlAtom = focusAtom(settingsAtom, (optic) =>
  optic.prop("base_url")
);

// const settingsEffect = atomEffect((get, set) => {

// })

const someAtom = withAtomEffect(settingsAtom, (get, set) => {
  const settings = get(settingsAtom);
  console.log("settings", settings);
  commands.setConfig(settings).then((result) => {
    if (result.status === "ok") {
      set(settingsAtom, settings);
      //   console.log("Settings saved", result.data);
    } else {
      toast.error("Failed to save settings");
    }
  });
});

export const useSettings = () => {
  const [settings, setSettings] = useAtom(settingsAtom);
  const [baseUrl, setBaseUrl] = useAtom(baseUrlAtom);
  useAtom(someAtom);

  const loadConfig = async () => {
    try {
      const result = await commands.getConfig();
      if (result.status === "ok") {
        console.log("Loaded config", result.data);
        setSettings(result.data);
      } else {
        toast.error("Failed to load config");
      }
    } catch (error) {
      toast.error("Failed to load config");
    }
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    loadConfig();
  }, []);

  return { settings, baseUrl, setSettings, setBaseUrl };
};

// export const baseSettingsAtom = atom(
//   async (_get) => {
//     const result = await commands.getConfig();
//     if (result.status === "ok") {
//       return result.data;
//     }

//     return fallbackCrawlConfig;
//   },
//   async (get, set, update: CrawlConfig) => {
//     await commands.setConfig(update);
//     const result = await commands.getConfig();
//     if (result.status === "ok") {
//       return result.data;
//     }
//   }
// );

// export const settingsAtom = unwrap(baseSettingsAtom, (prev) => {
//   if (prev) {
//     return prev;
//   }
//   return fallbackCrawlConfig;
// });

// atom<CrawlConfig>({
//   base_url: "",
//   max_concurrent_requests: 10,
//   request_delay_ms: 1000,
// });
