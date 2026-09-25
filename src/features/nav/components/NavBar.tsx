import { M3eNavBar, M3eNavItem } from "@m3e/react/nav-bar";
import { M3eIcon } from "@m3e/react/icon";
import style from "./NavBar.module.css";
import "@m3e/icons/outlined/home";
import "@m3e/icons/outlined/looks_6";
import "@m3e/icons/outlined/calendar_today";
import "@m3e/icons/outlined/mail";

function NavBar() {
    return (
        <M3eNavBar className={style.navBar}>
            <M3eNavItem selected>
                <M3eIcon slot="icon" name="home" />
                Home
            </M3eNavItem>
            <M3eNavItem>
                <M3eIcon slot="icon" name="looks_6" />
                Grades
            </M3eNavItem>
            <M3eNavItem>
                <M3eIcon slot="icon" name="calendar_today" />
                Timetable
            </M3eNavItem>
            <M3eNavItem>
                <M3eIcon slot="icon" name="mail" />
                Messages
            </M3eNavItem>
        </M3eNavBar>
    )
}

export default NavBar
