import { useLatestGrades } from "../hooks";
import SkeletonLoader from "../../../components/SkeletonLoader";
import CardList from "../../../components/CardList";
import { M3eHeading } from "@m3e/react/heading";


function GradeList() {
    const { isLoading, error, data } = useLatestGrades()
    if (error != null)
        throw error

    const component = isLoading || data === undefined
        ? <SkeletonLoader width="4rem" height="4rem" />
        : <CardList items={
            data.map(grade => {
                return {
                    id: grade.id,
                    props: {
                        leading: <M3eHeading variant="headline" size="large">{grade.grade}</M3eHeading>,
                        title: grade.subject.name,
                        subtitle: grade.category.name,
                    }
                }
            })
        } />
    return component
}

export default GradeList
