import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
    const [output, setOutput] = useState("");
    const [login, setLogin] = useState("");
    const [password, setPassword] = useState("");

    async function send() {
        setOutput(await invoke("send", { login, password }));
    }

    return (
        <main className="container">
            <h1>Welcome to Tauri + React</h1>

            <div className="row">
                <a href="https://vite.dev" target="_blank">
                    <img src="/vite.svg" className="logo vite" alt="Vite logo" />
                </a>
                <a href="https://tauri.app" target="_blank">
                    <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
                </a>
                <a href="https://react.dev" target="_blank">
                    <img src={reactLogo} className="logo react" alt="React logo" />
                </a>
            </div>
            <p>Click on the Tauri, Vite, and React logos to learn more.</p>

            <form
                className="row"
                onSubmit={(e) => {
                    e.preventDefault();
                    send();
                }}
            >
                <input
                    id="login-input"
                    onChange={(e) => setLogin(e.currentTarget.value)}
                    placeholder="Login..."
                />
                <input
                    id="password-input"
                    onChange={(e) => setPassword(e.currentTarget.value)}
                    placeholder="Password..."
                />
                <button type="submit">Send</button>
            </form>
            <p>{output}</p>
        </main>
    );
}

export default App;
