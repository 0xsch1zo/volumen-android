import { useQuery } from "@tanstack/react-query";
import { gradesList } from "./api";

function useLatestGrades() {
    const MAX_GRADES_DISPLAYED = 3
    return useQuery({
        queryKey: ["latestGrades"],
        queryFn: async () => {
            let grades = await gradesList()
            grades.sort((a, b) => {
                let aDate = new Date(a.date)
                let bDate = new Date(b.date)
                if (aDate < bDate) {
                    return -1
                } else if (aDate == bDate) {
                    return 0
                } else { return 1 }
            })

            if (grades.length >= 1) {
                let maxDisplayed = (grades.length < MAX_GRADES_DISPLAYED)
                    ? grades.length
                    : MAX_GRADES_DISPLAYED;
                return grades.slice(grades.length - maxDisplayed)
            } else {
                return grades
            }
        },
    })
}

export {
    useLatestGrades,
}
