import { Outlet } from "react-router-dom";

function Layout() {
  return (
    <div className="flex min-h-screen w-full flex-col">
      <Outlet />
    </div>
  )
}

export default Layout;
