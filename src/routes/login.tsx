import type { Route } from "./+types/login";

import LoginForm from "../features/login/components/LoginForm";
import GeometricBackground from "../components/GeometricBackground";
import style from "./login.module.css"


function LoginPage({ }: Route.ComponentProps) {
    return (
        <div className={style.container}>
            <GeometricBackground />
            <LoginForm />
        </div>
    )
}

export default LoginPage
