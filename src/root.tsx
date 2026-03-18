import "./App.css";
import { Outlet } from "react-router";

function App() {
    return (
        <main className="container">
            <Outlet />
        </main >
    );
}

export default App;
