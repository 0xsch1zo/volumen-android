import { useCallback, useState } from "react";
import { useNavigate } from "react-router";
import { MessageBox } from "../types";
import { useMessage } from "../hooks";
import ComposeReply from "./ComposeReply";
import style from "./MessageDetail.module.css";

function formatDate(value: string): string {
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    return new Intl.DateTimeFormat(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
    }).format(parsed);
}

function MessageDetail({ box, messageId }: { box: MessageBox; messageId: number }) {
    const navigate = useNavigate();
    const [replyOpen, setReplyOpen] = useState(false);
    const query = useMessage(box, messageId);

    const refreshAfterSend = useCallback(() => {
        setReplyOpen(false);
        void query.refetch();
    }, [query.refetch]);

    if (query.error != null) throw query.error;
    if (query.isLoading || query.data === undefined) {
        return <div className={style.loading}>Loading message…</div>;
    }

    const message = query.data;
    const primaryPerson = box === "received"
        ? message.sender_name
        : message.receivers.map((receiver) => receiver.name).join(", ");

    return (
        <article className={style.detail}>
            <button className={style.back} type="button" onClick={() => navigate(-1)}>
                ← Back
            </button>
            <div className={style.header}>
                <div>
                    <h1>{message.topic}</h1>
                    <p className={style.meta}>
                        {box === "received" ? "From" : "To"}: {primaryPerson || "—"}
                    </p>
                    <p className={style.meta}>Sent: {formatDate(message.send_date)}</p>
                    {message.read_date ? <p className={style.meta}>Read: {formatDate(message.read_date)}</p> : null}
                </div>
            </div>
            <div className={style.body}>{message.message}</div>

            {message.attachments.length > 0 ? (
                <section className={style.attachments}>
                    <h2>Attachments</h2>
                    {message.attachments.map((attachment) => (
                        <div key={attachment.id} className={style.attachment}>
                            {attachment.filename}
                        </div>
                    ))}
                </section>
            ) : null}

            {box === "received" && !message.no_reply ? (
                replyOpen ? (
                    <ComposeReply messageId={message.message_id} onSent={refreshAfterSend} />
                ) : (
                    <button className={style.reply} type="button" onClick={() => setReplyOpen(true)}>
                        Reply
                    </button>
                )
            ) : null}
        </article>
    );
}

export default MessageDetail;
