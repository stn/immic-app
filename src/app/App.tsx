import {
  BrowserRouter,
  Route,
  Routes,
} from "react-router-dom";
import { useKey } from "react-use";
import { appWindow } from '@tauri-apps/api/window';

import { ThemeProvider } from "@/components/theme-provider";

import Layout from "./Layout";
import MainPanel from "@/pages/MainPanel";
import Settings from "@/pages/Settings";

function App() {
  useKey("Escape", () => { appWindow.hide(); })

  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<MainPanel />} />
            <Route path="/settings" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
}

export default App;
