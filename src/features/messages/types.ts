export type MessageBox = "received" | "sent";

export interface AttachmentReference {
    id: number;
    filename: string;
}

export interface Receiver {
    receiver_id: number;
    name: string;
    read_date: string;
}

export interface ReceivedMessagePreview {
    message_id: number;
    sender_name: string;
    topic: string;
    fragment: string;
    send_date: string;
    read_date: string | null;
    has_file_attachment: boolean;
}

export interface SentMessagePreview {
    message_id: number;
    receiver_name: string;
    topic: string;
    fragment: string;
    send_date: string;
    has_file_attachment: boolean;
}

export interface ReceivedMessagesPage {
    messages: ReceivedMessagePreview[];
    total: number;
}

export interface SentMessagesPage {
    messages: SentMessagePreview[];
    total: number;
}

export interface ReceivedMessage {
    message_id: number;
    sender_name: string;
    topic: string;
    message: string;
    send_date: string;
    read_date: string | null;
    no_reply: boolean;
    is_archived: boolean;
    attachments: AttachmentReference[];
    receivers: Receiver[];
}

export interface SentMessage {
    message_id: number;
    sender_name: string;
    topic: string;
    message: string;
    send_date: string;
    read_date: string | null;
    no_reply: boolean;
    is_archived: boolean;
    attachments: AttachmentReference[];
    receivers: Receiver[];
}

export type MessageDetail = ReceivedMessage | SentMessage;
