import type { Route } from "./+types/login";

import LoginPage from "../pages/LoginPage";
import initTheme from "../theme";


export async function clientLoader({ }) {
    document.body.style.visibility = 'hidden'
    await initTheme().then(() => {
        document.body.style.visibility = 'visible'
    })
}

function Login({ }: Route.ComponentProps) {
    return <LoginPage />
}

export default Login;

