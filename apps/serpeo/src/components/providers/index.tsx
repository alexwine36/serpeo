import { ThemeProvider } from "@repo/ui/components/theme-provider";
import { Provider } from "jotai";
import { atomStore } from "../../atoms/store";
type ProvidersProps = {
  children: React.ReactNode;
};

export const Providers = ({ children }: ProvidersProps) => {
  return (
    <ThemeProvider>
      <Provider store={atomStore}>{children}</Provider>
    </ThemeProvider>
  );
};
