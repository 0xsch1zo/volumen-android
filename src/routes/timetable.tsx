import { full_timetable } from "../features/timetable/api"
import DateNavButtons from "../features/timetable/components/DateNavButtons"
import PeriodSwitch from "../features/timetable/components/PeriodSwitch"
import TimeblockList from "../features/timetable/components/TimeblockList"
import { Timetable } from "../features/timetable/types"
import { queryClient } from "../root"
import type { Route } from "./+types/timetable"
import style from "./timetable.module.css";

async function clientLoader(): Promise<Timetable> {
    return await queryClient.ensureQueryData({
        queryKey: ["fullTimetable"],
        queryFn: async () => await full_timetable(null),
    })
}

function TimetablePage({ loaderData }: Route.ComponentProps) {
    let timetable = loaderData
    console.log(loaderData)
    let current_date = new Date(Date.parse(timetable.date))
    return <div className={style.pageContainer}>
        <DateNavButtons current_date={current_date} />
        <div className={style.timeblockListContainer}>
            <TimeblockList day={timetable.days.find((day) => day.date == timetable.date)!} />
        </div>
        <PeriodSwitch />
    </div>
}

export { clientLoader }
export default TimetablePage
