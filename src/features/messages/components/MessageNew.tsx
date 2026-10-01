import { M3eFab } from "@m3e/react/fab";
import { M3eIcon } from "@m3e/react/icon";
import "@m3e/icons/outlined/add";

function MessageNew() {
    return <M3eFab aria-label="New message">
        <M3eIcon name="add" />
    </M3eFab>
}

export default MessageNew
