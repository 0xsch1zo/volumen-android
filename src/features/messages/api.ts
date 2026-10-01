import { invoke } from "@tauri-apps/api/core";
import { ReceivedMessagePreviews } from "./types";

async function receivedMessages(): Promise<ReceivedMessagePreviews> {
    return await invoke("recent_messages")
}

export { receivedMessages }
