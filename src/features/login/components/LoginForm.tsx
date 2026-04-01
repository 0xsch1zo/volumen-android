import { M3eButton } from "@m3e/react/button"
import { M3eFormField } from "@m3e/react/form-field"
import { M3eHeading } from "@m3e/react/heading"
import { useState } from "react"
import { login } from "../api.ts"
import { useNavigate } from "react-router"
import style from "./LoginForm.module.css"

function LoginForm() {
    const [loginValue, setLogin] = useState("");
    const [password, setPassword] = useState("");
    const navigate = useNavigate();

    return (
        <div className={style.container}>
            <M3eHeading
                variant="headline"
                size="large"
                className={style.heading}
                emphasized
            >
                Volumen
            </M3eHeading>

            <form
                className={style.form}
                onSubmit={async (e) => {
                    e.preventDefault()
                    await login(loginValue, password)
                    navigate("/choose-account")
                }}
            >
                <M3eFormField
                    variant="outlined"
                >
                    <label slot="label" htmlFor="login-input">Login</label>
                    <input
                        id="login-input"
                        onChange={(e) => setLogin(e.currentTarget.value)}
                    ></input>
                </M3eFormField>

                <M3eFormField
                    variant="outlined"
                >
                    <label slot="label" htmlFor="password-input">Password</label>
                    <input
                        id="password-input"
                        type="password"
                        onChange={(e) => setPassword(e.currentTarget.value)}
                    ></input>
                </M3eFormField>

                <div className={style.actionButtons}>
                    <M3eButton
                        variant="filled"
                        shape="square"
                        size="medium"
                        type="submit"
                    >
                        Login
                    </M3eButton>

                    <M3eButton
                        variant="tonal"
                        shape="square"
                        size="medium"
                    >
                        Register
                    </M3eButton>
                </div>
            </form>
        </div>
    );
}

export default LoginForm;
