import { useLocation } from "react-router";
import MessageTabs from "../features/messages/components/MessagesTabs";
import MessagePreviewList from "../features/messages/components/MessagesPreviewList";
import style from "./message.module.css";

function MessagesPage() {
    const location = useLocation();
    const active = location.pathname === "/messages/sent" ? "sent" : "received";

    return (
        <div className={style.page}>
            <h1 className={style.title}>Messages</h1>
            <MessageTabs active={active} />
            <MessagePreviewList box={active} />
        </div>
    );
}

export default MessagesPage;
