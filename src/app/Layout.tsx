import { Link, Outlet } from "react-router-dom";
import { Settings } from "lucide-react";

function Layout() {
  return (
    <div className="flex min-h-screen w-full flex-col">
      <header className="sticky top-0 h-16 items-center gap-4 bg-transparent px-4">
        <nav className="flex gap-6 text-lg font-medium mt-2">
          <div className="ml-auto h-12 pt-2">
            <Link
              to="/settings"
              className="text-muted-foreground transition-colors hover:text-foreground ml-auto"
            >
              <Settings className="h-6 w-6 text-muted-foreground" />
            </Link>
          </div>
        </nav>
      </header>
      <main className="flex flex-1 flex-col gap-4 bg-background p-4">
        <Outlet />
      </main>
    </div>
  )
}

export default Layout;
