import { useEffect, useState } from "react";
import { Link } from "react-router-dom";

import { dialog } from "@tauri-apps/api";
import * as autostart from "tauri-plugin-autostart-api";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import {
  settingGet,
  settingLoad,
  settingSave,
  settingSet,
  quitApp
} from "@/lib/api";

function Settings() {
  const [dataDir, setDataDir] = useState("");
  const [serverPort, setServerPort] = useState("");
  const [watchPathset, setWatchPathset] = useState<string>("");
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [globalShortcut, setGlobalShortcut] = useState<string>("");

  function storePreferences() {
    (async () => {
      await settingSet("data-dir", dataDir);
      await settingSet("server-port", serverPort);
      await settingSet("watch-pathset", watchPathset);
      await settingSet("global-shortcut", globalShortcut);
      await settingSave();
      await quitApp();
    })();
  }

  function selectDataDir() {
    (async () => {
      const selected = await dialog.open({
        directory: true,
        defaultPath: dataDir,
      });
      if (selected && typeof selected === "string" && selected !== "" && dataDir !== selected) {
        setDataDir(selected);
      }
    })();
  }

  useEffect(() => {
    let isMounted = true;
    (async () => {
      await settingLoad();
      const dataDir = await settingGet<string>("data-dir") || "";
      const serverPort = await settingGet<string>("server-port") || "53294";
      const watchPathset = await settingGet<string>("watch-pathset") || "";
      const autostartEnabled = await autostart.isEnabled();
      const globalShortcut = await settingGet<string>("global-shortcut") || "Alt+Shift+K";
      if (isMounted) {
        setDataDir(dataDir);
        setServerPort(serverPort);
        setWatchPathset(watchPathset);
        setAutostartEnabled(autostartEnabled);
        setGlobalShortcut(globalShortcut);
      }
    })();
    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <>
      <div className="mx-auto grid w-full max-w-6xl items-start gap-6 md:grid-cols-[180px_1fr] lg:grid-cols-[250px_1fr]">
        <nav className="grid gap-4 text-sm text-muted-foreground">
          <h1 className="text-3xl font-semibold">
            Settings
          </h1>
          <Link to="#" className="font-semibold text-primary">
            General
          </Link>
          <Link to="#">Advanced</Link>
        </nav>
        <div className="grid gap-4">
          <form>
            <div className="mx-auto grid max-w-[59rem] flex-1 auto-rows-max gap-4">
              <div className="grid gap-6">
                <div className="flex gap-6">
                  {/* <h1 className="flex-1 shrink-0 whitespace-nowrap text-xl font-semibold tracking-tight sm:grow-0">
                    General
                  </h1> */}
                  <div className="items-center gap-2 ml-auto">
                    <Button
                      size="sm"
                      onClick={(e) => {
                        e.preventDefault();
                        storePreferences();
                      }}
                    >
                      Save
                    </Button>
                  </div>
                </div>
                <div className="grid gap-4">
                  <div className="grid auto-rows-max items-start gap-4 lg:col-span-2 lg:gap-8">
                    <Card>
                      <CardHeader>
                        <CardTitle>General</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <div className="grid gap-6">
                          <div className="grid gap-3">
                            <Label htmlFor="data-dir">Data Directory</Label>
                            <div>
                              {dataDir}
                            </div>
                            <Button
                              id="data-dir"
                              variant="outline"
                              onClick={(e) => {
                                e.preventDefault();
                                selectDataDir()
                              }}
                            >
                              Select
                            </Button>
                          </div>
                          <div className="grid gap-3">
                            <Label htmlFor="server-port">Server Port</Label>
                            <Input
                              id="server-port"
                              type="number"
                              className="w-full"
                              defaultValue={serverPort}
                              onChange={(e) => setServerPort(e.currentTarget.value)}
                            />
                          </div>
                          <div className="grid gap-3">
                            <Label htmlFor="watch-pathset">Watch Pathset</Label>
                            <Input
                              id="watch-pathset"
                              type="text"
                              className="w-full"
                              defaultValue={watchPathset}
                              onChange={(e) => setWatchPathset(e.currentTarget.value)}
                            />
                          </div>
                          <div className="grid gap-3">
                            <Label htmlFor="autostart">Autostart</Label>
                            <Checkbox
                              id="autostart"
                              defaultChecked={autostartEnabled}
                              onCheckedChange={(state) => {
                                if (state === true) {
                                  setAutostartEnabled(true);
                                  autostart.enable();
                                } else {
                                  setAutostartEnabled(false);
                                  autostart.disable();
                                }
                              }}
                            />
                          </div>
                          <div className="grid gap-3">
                            <Label htmlFor="global-shortcut">Global Shortcut</Label>
                            <Input
                              id="global-shortcut"
                              type="text"
                              defaultValue={globalShortcut}
                              onChange={(e) => setGlobalShortcut(e.currentTarget.value)}
                            />
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  </div>
                </div>
              </div>
            </div>
          </form>
        </div>
      </div>
    </>
  );
}

export default Settings;
