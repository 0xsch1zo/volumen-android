import { invoke } from "@tauri-apps/api/core";
import { ReceivedMessagePreviews, SentMessagePreviews } from "./types";

async function receivedMessages(): Promise<ReceivedMessagePreviews> {
    return await invoke("recent_messages")
}

async function sentMessages(): Promise<SentMessagePreviews> {
    return await invoke("sent_messages")
}

export { receivedMessages, sentMessages }
