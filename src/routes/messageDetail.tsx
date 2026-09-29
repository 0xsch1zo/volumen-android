import type { Route } from "./+types/messageDetail";
import MessageDetail from "../features/messages/components/MessageDetail";
import { MessageBox } from "../features/messages/types";
import { messageDetail } from "../features/messages/api";
import { queryClient } from "../root";
import style from "./messageDetail.module.css";

function parseBox(value: string | undefined): MessageBox {
    if (value === "received" || value === "sent") return value;
    throw new Response("Unknown message box", { status: 404 });
}

function parseMessageId(value: string | undefined): number {
    const id = Number(value);
    if (!Number.isInteger(id) || id <= 0) {
        throw new Response("Invalid message id", { status: 404 });
    }
    return id;
}

async function clientLoader({ params }: Route.ClientLoaderArgs) {
    const box = parseBox(params.type);
    const messageId = parseMessageId(params.messageId);

    return await queryClient.ensureQueryData({
        queryKey: ["message", box, messageId],
        queryFn: () => messageDetail(box)(messageId),
    });
}

function MessageDetailPage({ params }: Route.ComponentProps) {
    const box = parseBox(params.type);
    const messageId = parseMessageId(params.messageId);

    return (
        <div className={style.page}>
            <MessageDetail box={box} messageId={messageId} />
        </div>
    );
}

export { clientLoader };
export default MessageDetailPage;
