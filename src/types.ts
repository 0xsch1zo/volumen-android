class Account {
    id: number
    group: string
    student_name: string

    constructor(id: number, student_name: string, group: string) {
        this.id = id
        this.student_name = student_name
        this.group = group
    }
}

export {
    Account
}
