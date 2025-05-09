import { Toaster } from "@repo/ui/components/sonner";
import { Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Providers } from "../components/providers";

export const Route = createRootRoute({
  component: () => (
    <>
      <Providers>
        {/* <NavBar /> */}

        {/* <div className="m-2"> */}
        <Outlet />
        {/* </div> */}
        <Toaster />

        <TanStackRouterDevtools />
      </Providers>
    </>
  ),
});
