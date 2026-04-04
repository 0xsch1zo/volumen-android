import { invoke } from "@tauri-apps/api/core";
import { Grade } from "../../types";

async function gradesList(): Promise<Array<Grade>> {
    return await invoke("grades_list")
}

export {
    gradesList
}
