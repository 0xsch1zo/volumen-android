import { invoke } from "@tauri-apps/api/core";
import { Grade, ReceivedMessagePreviews } from "../../types";
import { DailyTimetable } from "./types";

async function gradesList(): Promise<Array<Grade>> {
    return await invoke("grades_list")
}

async function dailyTimetable(): Promise<DailyTimetable> {
    return await invoke("daily_timetable")
}

async function recentMessages(): Promise<ReceivedMessagePreviews> {
    return await invoke("recent_messages")
}

export {
    gradesList,
    dailyTimetable,
    recentMessages
}
