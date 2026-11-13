import { M3eActionList, M3eListAction } from "@m3e/react/list";
import { MessagePreview } from "../types";
import "@m3e/icons/outlined/arrow_right";
import "@m3e/icons/outlined/star";
import style from "./MessageList.module.css";
import { M3eDivider } from "@m3e/react/divider";
import { M3eHeading } from "@m3e/react/heading";

function formatDate(date: string): string {
    const parsed = new Date(date)
    return isNaN(parsed.getTime()) ? date : parsed.toLocaleDateString()
}
function personOf(message: MessagePreview): string {
    return "sender_name" in message ? message.sender_name : message.receiver_name
}

function MessageItem({ message, last }: { message: MessagePreview; last: boolean }) {
    return <>
        <M3eListAction className={style.item}>
            <span slot="overline">{personOf(message)}</span>
            <div className={style.header}>
                {message.topic}
                <M3eHeading className={style.date} variant="label" size="large">{formatDate(message.send_date)}</M3eHeading>
            </div>
            <span slot="supporting-text" >{message.fragment}</span>
        </M3eListAction>
        {last ? null : <M3eDivider inset />}
    </>
}

function MessageList({ messages }: { messages: Array<MessagePreview> }) {
    if (messages.length === 0) {
        return <div className={style.empty}>No messages</div>
    }

    return <M3eActionList>
        {messages.map((message, i) =>
                <MessageItem 
                    key={message.message_id}    
                    message={message}
                    last={i === messages.length - 1}
                />
        )}
    </M3eActionList>
}

export default MessageList

