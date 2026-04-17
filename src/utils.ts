import { invoke } from "@tauri-apps/api/core";
import { Account } from "./types";

async function currentAccount(): Promise<Account> {
    return await invoke("current_account", {})
}

export {
    currentAccount,
}
