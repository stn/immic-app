import {
  BrowserRouter,
  Route,
  RouterProvider,
  Routes,
} from "react-router-dom";

import { ThemeProvider } from "@/components/theme-provider";

import Layout from "./Layout";
import Dashboard from "./Dashboard";
import Pref from "./Pref";

function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="ui-theme">
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/setting" element={<Pref />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  );
}

export default App;
