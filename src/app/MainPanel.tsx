import { useEffect, useState } from "react";
import { Link, Outlet, useNavigate } from "react-router-dom";
import { useForm, SubmitHandler } from "react-hook-form";

import * as GS from "@tauri-apps/api/globalShortcut";

import { Settings } from "lucide-react";
import { Input } from "@/components/ui/input"

import {
  settingGet,
  settingLoad,
  showMain,
} from "@/lib/api";

type SearchInputs = {
  query: string;
}

function MainPanel() {
  const navigate = useNavigate();

  const [dataDir, setDataDir] = useState("");
  const [globalShortcut, setGlobalShortcut] = useState<string>();
  const [shortcutRegistered, setShortcutRegistered] = useState<boolean>(false);

  const { register, handleSubmit } = useForm<SearchInputs>();
  const onSubmit: SubmitHandler<SearchInputs> = async (data) => {
    navigate("/search", { state: { query: data.query } });
  };

  useEffect(() => {
    let isMounted = true;

    (async () => {
      // Check if data-dir is set
      if (dataDir === "") {
        await settingLoad();
        const dataDir = await settingGet<string>("data-dir") || "";
        if (isMounted) {
          setDataDir(dataDir);
          if (dataDir === "") {
            navigate("/settings");
            return;
          }
        }
      }

      // Register global shortcut
      if (!shortcutRegistered) {
        const shortcut = await settingGet<string>("global-shortcut") || "Alt+Shift+K";
        if (isMounted) {
            await GS.register(shortcut, () => {
              showMain();
            });
            console.log('registered shortcut: ', shortcut)
            setGlobalShortcut(shortcut);
            setShortcutRegistered(true);
        }
      }

    })();

    return () => {
      isMounted = false;
      globalShortcut && GS.unregister(globalShortcut);
    };
  }, []);

  return (
    <>
      <header className="sticky top-0 h-16 items-center bg-transparent px-4">
        <nav className="flex gap-6 text-lg font-medium mt-2">
          <div className="ml-[550px] w-[1000px]">
            <form onSubmit={handleSubmit(onSubmit)}>
              <Input
                type="search"
                className="bg-transparent focus:bg-background pl-8"
                {...register("query")}
              />
              <input type="submit" className="hidden" />
            </form>
          </div>
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
      <main className="flex flex-1 flex-col gap-4 bg-background pl-4 pr-4">
        <Outlet />
      </main>
    </>
  );
}

export default MainPanel;
