import SubjectGradesList from "../features/grades/components/SubjectGradesList";
import type { Route } from "./+types/grades";

function GradesPage({ }: Route.ComponentProps) {
    return <SubjectGradesList />
}

export default GradesPage
