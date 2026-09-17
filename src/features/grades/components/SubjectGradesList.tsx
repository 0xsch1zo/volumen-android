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
    return (
        <M3eChipSet className={styles.gradeChipList}>
            {grades.map(grade => <M3eChip>{grade.grade}</M3eChip>)}
        </M3eChipSet>
    )
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
