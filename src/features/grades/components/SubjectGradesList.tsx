import { Grade } from "../../../types";
import { SubjectGrades } from "../types";
import { M3eDivider } from "@m3e/react/divider";
import { M3eCard } from "@m3e/react/card";
import { M3eHeading } from "@m3e/react/heading";
import { M3eChip, M3eChipSet } from "@m3e/react/chips";
import { useQuery } from "@tanstack/react-query";
import { subjectGradesList } from "../api";
import styles from "./SubjectGradesList.module.css";

function GradeChipList({ grades }: { grades: Array<Grade> }) {
    let grades_val = grades.map(grade => grade.grade).concat(["5"
        , "6", "4", "4+"]); // TEMP 
    let element;

    if (grades_val.length != 0) {
        element = <M3eChipSet className={styles.gradeChipList}>
            {grades_val.map(grade => <M3eChip className={styles.gradeChip}><M3eHeading variant="title" size="medium">{grade}</M3eHeading></M3eChip>)}
        </M3eChipSet>
    } else {
        element = <div className={styles.emptySubjectGrades}><M3eHeading variant="label" size="medium">-</M3eHeading></div>
    }
    return element
}

function SubjectGradesCard({ subjectGrades }: { subjectGrades: SubjectGrades }) {
    return (
        <M3eCard variant="outlined">
            <GradeChipList grades={subjectGrades.grades} />
            <M3eDivider />
            <M3eHeading variant="title" size="small" className={styles.subjectName}>{subjectGrades.subject.name}</M3eHeading>
        </M3eCard>
    )
}


function SubjectGradesList() {
    const { isLoading, error, data } = useQuery({
        queryKey: ["subjectGradesList"],
        queryFn: subjectGradesList,
    })
    if (error != null)
        throw error

    if (!isLoading && data !== undefined) {
        return <>{data.map(subjectGrades => <SubjectGradesCard subjectGrades={subjectGrades} />)}</>
    }
}

export default SubjectGradesList
