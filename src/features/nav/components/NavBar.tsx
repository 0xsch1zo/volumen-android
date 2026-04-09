import { M3eNavBar, M3eNavItem } from "@m3e/react/nav-bar";
import homeIcon from "../assets/home.svg"
import gradeIcon from "../assets/grade.svg"
import calendarIcon from "../assets/calendar.svg"
import messageIcon from "../assets/message.svg"

/*function Item() {
    let navItem = <M3eNavItem />
    navItem.props.children;
}*/

function NavBar() {
    return (
        <M3eNavBar>
            <M3eNavItem selected>
                <img slot="icon" src={homeIcon} />
                Home
            </M3eNavItem>
            <M3eNavItem>
                <img slot="icon" src={gradeIcon} />
                Grades
            </M3eNavItem>
            <M3eNavItem>
                <img slot="icon" src={calendarIcon} />
                Timetable
            </M3eNavItem>
            <M3eNavItem>
                <img slot="icon" src={messageIcon} />
                Messages
            </M3eNavItem>
        </M3eNavBar>
    )
}

export default NavBar
