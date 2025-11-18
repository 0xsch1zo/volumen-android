use serde::{Deserialize, Serialize};

use crate::repositories::entities::{CategoryId, LessonId, StudentId, SubjectId};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShallowGrades {
    grades: Vec<ShallowGrade>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShallowGrade {
    lesson: LessonId,
    subject: SubjectId,
    student: StudentId,
    category: CategoryId,
    grade: String,
    date: String,
    add_date: String,
    semester: usize,
    is_constituent: bool,
    is_semester: bool,
    is_semester_proposition: bool,
    is_final: bool,
    is_final_proposition: bool,
}

