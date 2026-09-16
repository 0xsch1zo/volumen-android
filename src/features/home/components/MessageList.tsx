import CardList from "../../../components/CardList"
import { useLatestMessages } from "../hooks"
import SkeletonLoader from "../../../components/SkeletonLoader"

function MessageList() {
    const { isLoading, error, data } = useLatestMessages()
    if (error != null)
        throw error

    if (isLoading || data === undefined)
        return <SkeletonLoader width="4rem" height="4rem" />
    else {
        return <CardList items={
            data.map(message => {
                return {
                    key: message.message_id,
                    props: {
                        header: message.sender_name,
                        title: message.topic,
                        subtitle: `${message.fragment}...`,
                    }
                }
            })
        } />

    }
}

export default MessageList
