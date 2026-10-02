import { M3eTabs, M3eTab } from "@m3e/react/tabs";

function MessageTabs() {
    return <M3eTabs variant="primary" stretch>
        <M3eTab selected>Received</M3eTab>
        <M3eTab>Sent</M3eTab>
    </M3eTabs>
}

export default MessageTabs 
