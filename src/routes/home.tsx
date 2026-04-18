import type { Route } from "./+types/home"
import GradeList from "../features/home/components/GradeList"
import style from "./home.module.css"
import LinkedHeader from "../features/home/components/LinkedHeader"

function HomePage({ }: Route.ComponentProps) {
    return (
        <div className={style.dashboardContainer}>
            <LinkedHeader
                title="Grades"
                destination="/temp"
            />
            <GradeList />
        </div>
    )
}

export default HomePage
