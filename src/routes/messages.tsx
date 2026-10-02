import { receivedMessages } from "../features/messages/api"
import MessageSearch from "../features/messages/components/MessageSearch"
import MessageTabs from "../features/messages/components/MessageTabs"
import MessageList from "../features/messages/components/MessageList"
import { ReceivedMessagePreviews } from "../features/messages/types"
import { queryClient } from "../root"
import type { Route } from "./+types/messages"
import style from "./messages.module.css"

async function clientLoader(): Promise<ReceivedMessagePreviews> {
    return await queryClient.ensureQueryData({
        queryKey: ["receivedMessages"],
        queryFn: receivedMessages,
    })
}

function MessagePage({ loaderData }: Route.ComponentProps) {
    return <div className={style.container}>
        <MessageTabs />
        <MessageList messages={loaderData.messages} />
        <MessageSearch />
    </div>
}

export { clientLoader }
export default MessagePage
