import { NavLink } from "react-router";
import style from "./MessageTabs.module.css";
import { MessageBox } from "../types";

function MessageTabs({ active }: { active: MessageBox }) {
    return (
        <div className={style.tabs} role="tablist" aria-label="Messages">
            <NavLink
                to="/messages"
                end
                role="tab"
                aria-selected={active === "received"}
                className={({ isActive }) => `${style.tab} ${isActive ? style.active : ""}`}
            >
                Received
            </NavLink>
            <NavLink
                to="/messages/sent"
                end
                role="tab"
                aria-selected={active === "sent"}
                className={({ isActive }) => `${style.tab} ${isActive ? style.active : ""}`}
            >
                Sent
            </NavLink>
        </div>
    );
}

export default MessageTabs;
