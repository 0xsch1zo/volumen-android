import { M3eButton } from "@m3e/react/button";
import { M3eFormField } from "@m3e/react/form-field";
import { M3eHeading } from "@m3e/react/heading";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

function LoginPage() {
    const [login, setLogin] = useState("");
    const [password, setPassword] = useState("");
    const [output, setOutput] = useState("");

    async function send() {
        setOutput(await invoke("send", { login, password }))
    }

    return (
        <div className="loginPage">
            <div className="pageBackground">
                <div className="pageBackground-6-sided-cookie"></div>
                <div className="pageBackground-9-sided-cookie"></div>
                <div className="pageBackground-circle"></div>
                <div className="pageBackground-pentagon"></div>
            </div>
            <div className="loginFormContainer">
                <M3eHeading
                    variant="headline"
                    size="large"
                    className="loginFormContainer-heading"
                    emphasized
                >
                    Volumen
                </M3eHeading>

                <form
                    className="loginFormContainer-form"
                    onSubmit={(e) => {
                        e.preventDefault()
                        send()
                    }}
                >
                    <M3eFormField variant="outlined">
                        <label slot="label" htmlFor="login-input">Login</label>
                        <input
                            id="login-input"
                            onChange={(e) => setLogin(e.currentTarget.value)}
                        ></input>
                    </M3eFormField>

                    <M3eFormField variant="outlined">
                        <label slot="label" htmlFor="password-input">Password</label>
                        <input
                            id="password-input"
                            type="password"
                            onChange={(e) => setPassword(e.currentTarget.value)}
                        ></input>
                    </M3eFormField>

                    <div className="loginFormContainer-buttons">
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
                <p>
                    {output}
                </p>
            </div>
        </div>
    );
}

export default LoginPage;
