import { TimeBlock } from "../../types";

export interface Timetable {
    date: string,
    days: Array<Day>,
}

export interface Day {
    date: String,
    time_blocks: Array<TimeBlock | null>,
}
