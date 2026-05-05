import type { Route } from "./+types/home"
import GradeList from "../features/home/components/GradeList"
import style from "./home.module.css"
import LinkedHeader from "../features/home/components/LinkedHeader"
import DailyTimetable from "../features/home/components/DailyTimetable"

function HomePage({ }: Route.ComponentProps) {
    return (
        <div className={style.dashboardContainer}>
            <LinkedHeader
                title="Grades"
                destination="/temp"
            />
            <GradeList />
            <LinkedHeader
                title="Timetable"
                destination="/temp"
            />
            <DailyTimetable />
        </div>
    )
}

export default HomePage
