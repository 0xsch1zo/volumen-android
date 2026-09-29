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
    } | null
    add_date: string
}

export interface ReceivedMessagePreview {
    // FIXME: change the name of this field to reflect the rest
    message_id: number,
    sender_name: string,
    topic: string,
    fragment: string,
    send_date: string,
    read_date?: string | null,
    has_file_attachment: boolean,
}

export interface ReceivedMessagePreviews {
    messages: Array<ReceivedMessagePreview>,
    total: number
}

export interface TimeBlock {
    start: String,
    end: String,
    subject: String,
    events: Array<Event>,
}
// new:
export interface SentMessagePreview {
    message_id: number,
    receiver_name: string,
    topic: string,
    fragment: string,
    send_date: string,
    has_file_attachment: boolean,
}

export interface SentMessagePreviews {
    messages: Array<SentMessagePreview>,
    total: number,
}

export interface AttachmentReference {
    id: number,
    filename: string,
}

export interface MessageReceiver {
    receiver_id: number,
    name: string,
    read_date: string,
}

export interface ReceivedMessage {
    message_id: number,
    sender_name: string,
    sender_id: number,   // for reply
    topic: string,
    message: string,
    send_date: string,
    read_date?: string | null,
    no_reply: boolean,
    is_archived: boolean,
    attachments: Array<AttachmentReference>,
    receivers: Array<MessageReceiver>,
}

export interface SentMessage {
    message_id: number,
    sender_name: string,
    topic: string,
    message: string,
    send_date: string,
    read_date?: string | null,
    no_reply: boolean,
    is_archived: boolean,
    attachments: Array<AttachmentReference>,
    receivers: Array<MessageReceiver>,
}