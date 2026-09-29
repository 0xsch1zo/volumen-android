import { useEffect, useMemo, useRef } from "react";
import CardList from "../../../components/CardList";
import SkeletonLoader from "../../../components/SkeletonLoader";
import {
    MessageBox,
    ReceivedMessagePreview,
    SentMessagePreview,
} from "../types";
import { useMessages } from "../hooks";
import { useNavigate } from "react-router";
import style from "./MessagePreview.module.css";

type MessageSummary = ReceivedMessagePreview | SentMessagePreview;

function formatDate(value: string): string {
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return value;
    return new Intl.DateTimeFormat(undefined, {
        day: "2-digit",
        month: "2-digit",
        year: "numeric",
    }).format(parsed);
}

function MessagePreviewList({ box }: { box: MessageBox }) {
    const navigate = useNavigate();
    const query = useMessages(box);
    const sentinelRef = useRef<HTMLDivElement | null>(null);

    const messages = useMemo<MessageSummary[]>(
        () => query.data?.pages.flatMap((page): MessageSummary[] => page.messages) ?? [],
        [query.data],
    );

    useEffect(() => {
        const element = sentinelRef.current;
        if (element === null) return;

        const observer = new IntersectionObserver(
            (entries) => {
                if (!entries[0]?.isIntersecting) return;
                if (!query.hasNextPage || query.isFetchingNextPage) return;
                void query.fetchNextPage();
            },
            { rootMargin: "320px 0px" },
        );

        observer.observe(element);
        return () => observer.disconnect();
    }, [query.fetchNextPage, query.hasNextPage, query.isFetchingNextPage]);

    if (query.error != null) throw query.error;

    if (query.isLoading) {
        return <SkeletonLoader width="100%" height="5rem" />;
    }

    return (
        <>
            {messages.length === 0 ? (
                <div className={style.empty}>No messages.</div>
            ) : (
                <CardList
                    items={messages.map((message) => ({
                        key: message.message_id,
                        props: {
                            header: box === "received"
                                ? "sender_name" in message
                                    ? message.sender_name
                                    : ""
                                : "receiver_name" in message
                                    ? message.receiver_name
                                    : "",
                            title: message.topic,
                            subtitle: `${message.fragment}${message.fragment.length > 0 ? "..." : ""} · ${formatDate(message.send_date)}`,
                            onAction: () => navigate(`/messages/${box}/${message.message_id}`),
                        },
                    }))}
                />
            )}
            <div ref={sentinelRef} className={style.sentinel} aria-hidden="true" />
            {query.isFetchingNextPage ? <div className={style.loadingMore}>Loading…</div> : null}
        </>
    );
}

export default MessagePreviewList;
