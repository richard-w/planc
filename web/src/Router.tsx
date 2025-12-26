import * as reactRouterDom from "react-router-dom";
import { LoginPage } from "./pages/Login.page";
import { SessionPage } from "./pages/Session.page";

const router = reactRouterDom.createBrowserRouter([
  {
    path: "/",
    element: <SessionPage />,
  },
  {
    path: "/login",
    element: <LoginPage />,
  },
]);

export function Router() {
  return <reactRouterDom.RouterProvider router={router} />;
}
