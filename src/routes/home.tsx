import type { Route } from "./+types/homeLayout"
import GradeList from "../features/home/components/GradeList"

function HomePage({ }: Route.ComponentProps) {
    console.log("should be fucking rendered")
    return (
        <>
            <GradeList />
        </>
    )
}

export default HomePage
