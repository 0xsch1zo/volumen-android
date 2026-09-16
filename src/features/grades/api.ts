import { invoke } from "@tauri-apps/api/core";
import { SubjectGrades } from "./types";

async function subjectGradesList(): Promise<Array<SubjectGrades>> {
    return await invoke("subject_grades_list")
}

export {
    subjectGradesList
}
