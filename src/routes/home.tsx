import type { Route } from "./+types/homeLayout"
import GradeList from "../features/home/components/GradeList"

function HomePage({ }: Route.ComponentProps) {
    return (
        <>
            <GradeList />
        </>
    )
}

export default HomePage
