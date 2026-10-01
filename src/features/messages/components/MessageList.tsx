import { M3eActionList, M3eListAction } from "@m3e/react/list";
import { ReceivedMessagePreview } from "../types";
import "@m3e/icons/outlined/arrow_right";
import "@m3e/icons/outlined/star";
import style from "./MessageList.module.css";

function formatDate(date: string): string {
    const parsed = new Date(date)
    return isNaN(parsed.getTime()) ? date : parsed.toLocaleDateString()")
}

function MessageList({ messages }: { messages: Array<ReceivedMessagePreview> }) {
    if (messages.length === 0) {
        return <div className={style.empty}>No messages</div>
    }

    return <M3eActionList>
        {messages.map((message) =>
            <M3eListAction key={message.message_id}>
                {message.topic}
                <span slot="supporting-text">{message.sender_name}</span>
                <div slot="trailing" className={style.trailing}>
                    <span>{formatDate(message.send_date)}</span>
                </div>
            </M3eListAction>
        )}
    </M3eActionList>
}

export default MessageList

