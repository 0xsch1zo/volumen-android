import type { Route } from "./+types/home"
import GradeList from "../features/home/components/GradeList"
import style from "./home.module.css"
import LinkedHeader from "../features/home/components/LinkedHeader"
import DailyTimetable from "../features/home/components/DailyTimetable"
import MessageList from "../features/home/components/MessageList"

function HomePage({ }: Route.ComponentProps) {
    return (
        <div className={style.dashboardContainer}>
            <LinkedHeader
                title="Grades"
                destination="/grades"
            />
            <GradeList />
            <LinkedHeader
                title="Timetable"
                destination="/timetable"
            />
            <DailyTimetable />
            <LinkedHeader
                title="Messages"
                destination="/temp"
            />
            <MessageList />
        </div>
    )
}

export default HomePage
