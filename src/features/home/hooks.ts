import { useQuery } from "@tanstack/react-query";
import { gradesList, recentMessages } from "./api";

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
                return grades.slice(grades.length - maxDisplayed).reverse()
            } else {
                return grades.reverse()
            }
        },
    })
}

function useLatestMessages() {
    const MAX_MESSAGES_DISPLAYED = 3
    return useQuery({
        queryKey: ["latestMessages"],
        queryFn: async () => {
            let messages = await recentMessages().then((messages) => messages.messages)
            if (messages.length >= 1) {
                let maxDisplayed = (messages.length < MAX_MESSAGES_DISPLAYED)
                    ? messages.length
                    : MAX_MESSAGES_DISPLAYED;
                // FIXME: i dunno why but slice doesnt work with (0, MAX_MESSAGES_DISPLAYED)
                return messages.slice(messages.length - maxDisplayed)
            } else {
                return messages
            }
        }
    })
}

export {
    useLatestGrades,
    useLatestMessages,
}
