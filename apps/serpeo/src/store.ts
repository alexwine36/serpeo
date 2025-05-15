import { useQuery } from "@tanstack/react-query";
import { Store } from "@tauri-apps/plugin-store";
import {
  CRAWL_SETTINGS_KEY,
  type CrawlSettingsStore,
  STORE_FILE,
} from "./generated/bindings";

let _store: Promise<Store> | undefined;
const store = () => {
  if (!_store) {
    _store = Store.load(STORE_FILE);
  }

  return _store;
};

function createTauriStore<T extends object>(name: string) {
  const get = () => store().then((s) => s.get<T>(name));
  const listen = (fn: (data?: T | undefined) => void) =>
    store().then((s) => s.onKeyChange<T>(name, fn));

  return {
    get,
    listen,
    set: async (value?: Partial<T>) => {
      console.log("set", value);
      const s = await store();
      if (value === undefined) {
        s.delete(name);
      } else {
        const current = (await s.get<T>(name)) || {};
        console.log("current", current);
        await s.set(name, {
          ...current,
          ...value,
        });
      }
      await s.save();
    },
    useQuery: () => {
      const query = useQuery({
        queryKey: ["store", name],
        queryFn: async () => (await get()) ?? null,
      });

      const _cleanup = listen(() => {
        query.refetch();
      });

      return query;
    },
  };
}

export const crawlSettingsStore =
  createTauriStore<CrawlSettingsStore>(CRAWL_SETTINGS_KEY);
