import { listen } from "@tauri-apps/api/event";

function registerLogoutListener() {
    listen<void>("logout", () => {
        console.log("LOGOUT SIGNAL");
    })
}

export default registerLogoutListener
