import { TimeBlock } from "../../types"

export interface DailyTimetable {
    day_of_week: String,
    when: "Today" | "Tomorrow" | "NextWeek",
    time_blocks: Array<TimeBlock>,
}

