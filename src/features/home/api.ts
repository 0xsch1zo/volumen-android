import { invoke } from "@tauri-apps/api/core";
import { Grade } from "../../types";
import { DailyTimetable } from "./types";

async function gradesList(): Promise<Array<Grade>> {
    return await invoke("grades_list")
}

async function dailyTimetable(): Promise<DailyTimetable> {
    return await invoke("daily_timetable")
}

export {
    gradesList,
    dailyTimetable,
}
