import { ThemeProvider } from "@repo/ui/components/theme-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Provider } from "jotai";
import { atomStore } from "../../atoms/store";
type ProvidersProps = {
  children: React.ReactNode;
};

export const Providers = ({ children }: ProvidersProps) => {
  const queryClient = new QueryClient();
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <Provider store={atomStore}>{children}</Provider>
      </ThemeProvider>
    </QueryClientProvider>
  );
};
