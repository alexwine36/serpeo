import { Outlet, createRootRoute } from "@tanstack/react-router";
export const Route = createRootRoute({
  component: () => (
    <div className="container mx-auto px-8 py-8 [view-transition-name:warp]">
      {/* <h1>Analysis</h1> */}
      <div>
        <Outlet />
      </div>
    </div>
  ),
});

// const AnalysisLayout = () => {
//     return (
//         <div>
//             <h1>Analysis</h1>
//             <Outlet />
//         </div>
//     )
// }
