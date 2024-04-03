import {
  BrowserRouter,
  Route,
  Routes,
} from "react-router-dom";

import { ThemeProvider } from "@/components/theme-provider";

import Layout from "./Layout";
import Dashboard from "../pages/Dashboard";
import Settings from "../pages/Settings";

function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/setting" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
}

export default App;
