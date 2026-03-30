import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { NavigateFunction } from "react-router";

async function registerLogoutHandler(navigate: NavigateFunction): Promise<UnlistenFn> {
    return await listen<void>("logout", () => {
        console.log("logout sigal")
        if (navigate !== undefined)
            navigate("/")
    })
}

export default registerLogoutHandler
