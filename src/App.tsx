import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [dates, setDates] = useState<string[]>([]);
  const [date, setDate] = useState<string>('');
  const [applications, setApplications] = useState([]);  // TODO set type
  const [screens, setScreens] = useState<string[]>([]);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  async function listDates() {
    setDates(await invoke("list_eventlog_dates"));
  }

  async function listApplications(date: string) {
    setApplications(await invoke("list_applications", { date }));
  }

  async function listScreens(date: string) {
    setScreens(await invoke("list_screens", { date }));
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
          listDates();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg}</p>
      <div>
        {dates.map((date) => (
          <button key={date} onClick={() => {
            setDate(date);
            listApplications(date);
            listScreens(date);
          }}>
            {date}
          </button>
        ))}
      </div>
      <div>
        {applications.map((app) => (
          <div key={app.id}>{app.id}: {app.name}</div>
        ))}
      </div>
      <div>
        {screens.map((screen) => (
          // <div key={screen}>{screen}</div>
          <img key={screen} src={'https://iss.localhost/' + date + '/' + screen + '-t'} alt={screen} />
        ))}
      </div>
    </div>
  );
}

export default App;
