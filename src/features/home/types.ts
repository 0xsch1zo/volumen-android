import { Event } from "../../types"

export interface TimeBlock {
    start: String,
    end: String,
    subject: String,
    events: Array<Event>,
}

export interface DailyTimetable {
    day_of_week: String,
    when: "Today" | "Tomorrow" | "NextWeek",
    time_blocks: Array<TimeBlock>,
}

