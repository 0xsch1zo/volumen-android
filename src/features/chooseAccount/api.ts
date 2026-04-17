import { invoke } from "@tauri-apps/api/core";
import { Account } from "../../types";


async function listAccounts(): Promise<Array<Account>> {
    return await invoke("accounts")
}

async function selectAccount(account: Account): Promise<void> {
    return await invoke("select_account", { account })
}

export {
    listAccounts,
    selectAccount,
}
