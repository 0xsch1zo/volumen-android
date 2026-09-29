import { FormEvent, useEffect, useState } from "react";
import { useSendMessage } from "../hooks";
import style from "./ComposeReply.module.css";

function ComposeReply({ messageId, onSent }: { messageId: number; onSent: () => void }) {
    const [value, setValue] = useState("");
    const mutation = useSendMessage();

    useEffect(() => {
        if (!mutation.isSuccess) return;
        setValue("");
        onSent();
        mutation.reset();
    }, [mutation.isSuccess, mutation.reset, onSent]);

    function submit(event: FormEvent<HTMLFormElement>) {
        event.preventDefault();
        const message = value.trim();
        if (message.length === 0 || mutation.isLoading) return;
        mutation.mutate({ messageId, message });
    }

    return (
        <form className={style.form} onSubmit={submit}>
            <label className={style.label} htmlFor="reply-message">Reply</label>
            <textarea
                id="reply-message"
                className={style.textarea}
                value={value}
                onChange={(event) => setValue(event.target.value)}
                placeholder="Write your reply…"
                rows={6}
                disabled={mutation.isLoading}
                required
            />
            {mutation.error instanceof Error ? (
                <p className={style.error}>{mutation.error.message}</p>
            ) : null}
            <button
                type="submit"
                className={style.submit}
                disabled={value.trim().length === 0 || mutation.isLoading}
            >
                {mutation.isLoading ? "Sending…" : "Send reply"}
            </button>
        </form>
    );
}

export default ComposeReply;
