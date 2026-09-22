import { full_timetable } from "../features/timetable/api"
import DateNavButtons from "../features/timetable/components/DateNavButtons"
import PeriodSwitch from "../features/timetable/components/PeriodSwitch"
import TimeblockList from "../features/timetable/components/TimeblockList"
import { Timetable } from "../features/timetable/types"
import { queryClient } from "../root"
import type { Route } from "./+types/timetable"

type LoaderData = {
    timetable: Timetable,
}

async function clientLoader(): Promise<LoaderData> {
    return await queryClient.ensureQueryData({
        queryKey: ["fullTimetable"],
        queryFn: async () => await full_timetable(null),
    })
}

function TimetablePage({ loaderData }: Route.ComponentProps) {
    // I dont' know what the hell is happening with this but right now it works
    let timetable = loaderData
    console.log(loaderData)
    let current_date = new Date(Date.parse(timetable.date))
    return <>
        <DateNavButtons current_date={current_date} />
        <TimeblockList day={timetable.days[current_date.getDay()]} />
        <PeriodSwitch />
    </>
}

export { clientLoader }
export default TimetablePage
