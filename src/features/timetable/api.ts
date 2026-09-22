import { invoke } from "@tauri-apps/api/core";
import { Timetable } from "./types";

async function full_timetable(date: string | null): Promise<Timetable> {
    return await invoke("full_timetable", { date })
}

export { full_timetable }
