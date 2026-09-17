import SubjectGradesList from "../features/grades/components/SubjectGradesList";
import type { Route } from "./+types/grades";
import styles from "./grades.module.css";

function GradesPage({ }: Route.ComponentProps) {
    return <div className={styles.list}><SubjectGradesList /></div>
}

export default GradesPage
