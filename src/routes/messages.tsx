import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { receivedMessages, sentMessages } from "../features/messages/api"
import MessageSearch from "../features/messages/components/MessageSearch"
import MessageTabs from "../features/messages/components/MessageTabs"
import MessageList from "../features/messages/components/MessageList"
import { MessageSource, ReceivedMessagePreviews } from "../features/messages/types"
import { queryClient } from "../root"
import type { Route } from "./+types/messages"
import style from "./messages.module.css"
import { M3eLinearProgressIndicator } from "@m3e/react/progress-indicator"
import { M3eHeading } from "@m3e/react/heading"

async function clientLoader(): Promise<ReceivedMessagePreviews> {
    return await queryClient.ensureQueryData({
        queryKey: ["receivedMessages"],
        queryFn: receivedMessages,
    })
}

function MessagePage({ loaderData }: Route.ComponentProps) {
    const [source, setSource] = useState<MessageSource>("received")

    const sent = useQuery({
        queryKey: ["sentMessages"],
        queryFn: sentMessages,
        enabled: source === "sent",
    })

    const handleTabChange = (tab: "received" | "sent") => {
        console.log("4. handleTabChange called with:", tab)
        setSource(tab)
    }

    let content
    if (source === "received") {
        content = <MessageList messages={loaderData.messages} />
    } else if (sent.isError) {
    content = <div className={style.loading}>
        <M3eHeading variant="label" size="large">Failed to load sent messages</M3eHeading>
    </div>
    } else if (sent.data == null) {
        content = <M3eLinearProgressIndicator mode="indeterminate" />
    } else {
        content = <MessageList messages={sent.data.messages} />
    }

    return <div className={style.container}>
        <MessageTabs source={source} onTabChange={handleTabChange} />
        <div className={style.list}>
            {content}
        </div>
        <MessageSearch />
    </div>
}

export { clientLoader }
export default MessagePage
