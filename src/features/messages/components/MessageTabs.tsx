import { M3eTabs, M3eTab } from "@m3e/react/tabs";
import { MessageSource } from "../types";

type TabElement = HTMLElement & { selected: boolean }

function MessageTabs({ source, onTabChange }: {
    source: MessageSource,
    onTabChange?: (tab: MessageSource) => void,
}) {
    function handleChange(event: Event) {
        const tabs = Array.from(
            (event.currentTarget as HTMLElement).querySelectorAll("m3e-tab")
        ) as Array<TabElement>

        const selected = tabs.find(tab => tab.selected)
        if (selected !== undefined)
            onTabChange?.(selected.dataset.source as MessageSource)
    }

    return <M3eTabs variant="primary" stretch onChange={handleChange}>
        <M3eTab selected={source === "received"} data-source="received">Received</M3eTab>
        <M3eTab selected={source === "sent"} data-source="sent">Sent</M3eTab>
    </M3eTabs>
}

export default MessageTabs