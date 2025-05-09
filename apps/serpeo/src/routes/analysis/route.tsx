import { Outlet, createRootRoute } from "@tanstack/react-router";
import { NavBar } from "../../components/layout/nav-bar";
export const Route = createRootRoute({
  component: () => (
    <div>
      <NavBar />
      <div className="container mx-auto px-8 py-8 [view-transition-name:warp]">
        {/* <h1>Analysis</h1> */}
        <div>
          <Outlet />
        </div>
      </div>
    </div>
  ),
});
