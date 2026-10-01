import { receivedMessages } from "../features/messages/api"
import MessageNew from "../features/messages/components/MessageNew"
import MessageSearch from "../features/messages/components/MessageSearch"
import MessageSwitch from "../features/messages/components/MessageSwitch"
import MessageList from "../features/messages/components/MessageList"
import { ReceivedMessagePreviews } from "../features/messages/types"
import { queryClient } from "../root"
import type { Route } from "./+types/messages"
import style from "./messages.module.css";

async function clientLoader(): Promise<ReceivedMessagePreviews> {
    return await queryClient.ensureQueryData({
        queryKey: ["receivedMessages"],
        queryFn: receivedMessages,
    })
}

const handle = { title: "Messages" }

function MessagePage({ loaderData }: Route.ComponentProps) {
    return <div className={style.pageContainer}>
        <MessageSwitch />
        <div className={style.messageListContainer}>
            <MessageList messages={loaderData.messages} />
        </div>
        <div className={style.bottomBar}>
            <MessageSearch />
            <MessageNew />
        </div>
    </div>
}

export { clientLoader, handle }
export default MessagePage
