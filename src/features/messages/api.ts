import { invoke } from "@tauri-apps/api/core";
import {
    MessageBox,
    ReceivedMessage,
    ReceivedMessagesPage,
    SentMessage,
    SentMessagesPage,
} from "./types";

const MESSAGE_PAGE_SIZE = 20;

async function receivedMessages(page: number, limit = MESSAGE_PAGE_SIZE): Promise<ReceivedMessagesPage> {
    return await invoke("received_messages", { page, limit });
}

async function sentMessages(page: number, limit = MESSAGE_PAGE_SIZE): Promise<SentMessagesPage> {
    return await invoke("sent_messages", { page, limit });
}

async function receivedMessage(messageId: number): Promise<ReceivedMessage> {
    return await invoke("received_message", { messageId });
}

async function sentMessage(messageId: number): Promise<SentMessage> {
    return await invoke("sent_message", { messageId });
}

async function sendMessage(messageId: number, message: string): Promise<void> {
    return await invoke("send_message", { messageId, message });
}

function messageList(box: MessageBox) {
    return box === "received" ? receivedMessages : sentMessages;
}

function messageDetail(box: MessageBox) {
    return box === "received" ? receivedMessage : sentMessage;
}

export {
    MESSAGE_PAGE_SIZE,
    receivedMessages,
    sentMessages,
    receivedMessage,
    sentMessage,
    sendMessage,
    messageList,
    messageDetail,
};
