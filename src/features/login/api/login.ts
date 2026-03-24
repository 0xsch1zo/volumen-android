import { invoke } from "@tauri-apps/api/core";

async function login(login: string, password: string) {
    await invoke("login", { login, password })
}

export {
    login
};
