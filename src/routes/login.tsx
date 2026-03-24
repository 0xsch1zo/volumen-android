import type { Route } from "./+types/login";

import LoginForm from "../features/login/components/LoginForm";
import initTheme from "../theme";
import PageBackground from "../features/login/components/PageBackground";
import style from "./login.module.css"


export async function clientLoader({ }) {
    document.body.style.visibility = 'hidden'
    await initTheme().then(() => {
        document.body.style.visibility = 'visible'
    })
}

function Login({ }: Route.ComponentProps) {
    return (
        <div className={style.container}>
            <PageBackground />
            <LoginForm />
        </div>
    )
}

export default Login;

