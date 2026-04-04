import { M3eNavBar, M3eNavItem } from "@m3e/react/nav-bar";

function NavBar() {
    return (
        <M3eNavBar>
            <M3eNavItem selected>
                <img slot="icon" src="../assets/home.svg" />
                Home
            </M3eNavItem>
            <M3eNavItem selected>
                <img slot="icon" src="../assets/grade.svg" />
                Grades
            </M3eNavItem>
            <M3eNavItem selected>
                <img slot="icon" src="../assets/calendar.svg" />
                Timetable
            </M3eNavItem>
            <M3eNavItem selected>
                <img slot="icon" src="../assets/message.svg" />
                Messages
            </M3eNavItem>
        </M3eNavBar>
    )
}

export default NavBar
