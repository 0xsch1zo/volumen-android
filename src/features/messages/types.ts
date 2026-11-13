import { ReceivedMessagePreview, SentMessagePreview } from "../../types";

export type { ReceivedMessagePreview, ReceivedMessagePreviews, SentMessagePreview, SentMessagePreviews } from "../../types";

export type MessageSource = "received" | "sent";

export type MessagePreview = ReceivedMessagePreview | SentMessagePreview;