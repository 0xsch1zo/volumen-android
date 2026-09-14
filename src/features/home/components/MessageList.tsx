import { M3eAvatar } from "@m3e/react/avatar"
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
                        // FIXME: sloppy letter picking
                        leading: <M3eAvatar>
                            {message.sender_name.at(0)}
                        </M3eAvatar>,
                        // TODO: seperate authors better from topics
                        title: `${message.topic} - ${message.sender_name}`,
                        subtitle: `${message.fragment}...`,
                    }
                }
            })
        } />

    }
}

export default MessageList
