export interface Account {
    id: number
    group: string
    student_name: string
}

export interface User {
    id: number,
    first_name: string,
    last_name: string,
}

export interface Subject {
    id: number
    name: string
}

export interface Category {
    id: number
    name: string
    weight: number
    count_to_the_avergae: boolean
}

export enum GradeKind {
    Constituent,
    Semester,
    SemesterPropsition,
    Final,
    FinalProposition,
    Unknown,
}

export interface Grade {
    id: number
    subject: Subject
    category: Category
    grade: string
    date: string
    add_date: string
    kind: GradeKind
}


export interface Event {
    id: number,
    content: string,
    date: string,
    category: {
        id: number,
        name: string
    },
    time_from: string,
    time_to: string,
    created_by: User,
    subject?: {
        id: number,
        name: string,
        short: string,
        is_extracurricular: string,
    }
    add_date: string
}
