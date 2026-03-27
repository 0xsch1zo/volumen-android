import type { Route } from "./+types/login";

import LoginForm from "../features/login/components/LoginForm";
import initTheme from "../theme";
import GeometricBackground from "../components/GeometricBackground";
import style from "./login.module.css"


async function clientLoader({ }) {
    document.body.style.visibility = 'hidden'
    await initTheme().then(() => {
        document.body.style.visibility = 'visible'
    })
}

function LoginPage({ }: Route.ComponentProps) {
    return (
        <div className={style.container}>
            <GeometricBackground />
            <LoginForm />
        </div>
    )
}

export default LoginPage
export { clientLoader }

