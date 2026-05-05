import { M3eHeading } from "@m3e/react/heading"
import { useQuery } from "@tanstack/react-query"
import { dailyTimetable } from "../api"
import { M3eDivider } from "@m3e/react/divider"
import { DailyTimetable as DailyTimetableDto } from "../types"
import { Event } from "../../../types"
import { M3eChip } from "@m3e/react/chips"
import style from "./DailyTimetable.module.css"

function DailyTimetable() {
    const { isLoading, error, data } = useQuery({
        queryKey: ["dailyTimetable"],
        queryFn: dailyTimetable,
    })
    if (error != null)
        throw error

    let element;
    if (isLoading)
        element = <div style={{ height: "14rem" }} />; // we need skeleton but it sort of doesn't work right now
    else {
        let when;
        switch (data?.when) {
            case "Today":
                when = "Today"
                break
            case "Tomorrow":
                when = "Tomorrow"
                break
            case "NextWeek":
                when = "Next week"
                break
        }

        let timetableElement;
        if (data !== undefined && data?.time_blocks.length != 0) {
            timetableElement = <>
                <Timeline timetable={data} />
                <div className={style.flexBoxVerticalDivider} />
                <SubjectEventList timetable={data} />
            </>
        } else {
            timetableElement = <div className={style.emptyTimetableOuterBox}>
                <div className={style.emptyTimetableInnerBox}>
                    <div className={style.emptyTimetableIcon} />
                    <M3eHeading variant="title" size="medium">Timetable is empty</M3eHeading>
                </div>
            </div >
        }


        element = (
            <div className={style.container}>
                <div className={style.upperContainer}>
                    <M3eHeading
                        className={style.title}
                        variant="title"
                        size="medium"
                    >
                        {data?.day_of_week}, {when?.toLowerCase()}
                    </M3eHeading>

                    <M3eHeading
                        variant="label"
                        size="small"
                        className={style.subtitle}
                    >
                        {
                            (data?.time_blocks.length == 0) ?
                                "No lessons here, enjoy your day!" :
                                <M3eHeading
                                    variant="label"
                                    size="medium">
                                    Starting off with
                                    <b className={style.secondaryWeightedText}>
                                        {` ${data?.time_blocks[0].subject}`}
                                    </b> at <b className={style.secondaryWeightedText}>
                                        {data?.time_blocks[0].start}
                                    </b>
                                </M3eHeading>
                        }
                    </M3eHeading>
                </div>
                <M3eDivider />
                <div className={style.timetableContainer}>
                    {timetableElement}
                </div>
            </div>
        )
    }

    return (
        element
    )
}

function Timeline({ timetable }: { timetable: DailyTimetableDto }) {
    return (
        <div className={style.timeline}>
            {timetable
                .time_blocks
                .map(t => (`${t.start} - ${t.end}`))
                .entries()
                .map(([k, v]: [number, string]) => <M3eHeading key={k} variant="label" size="medium">{v}</M3eHeading>)}
        </div>
    )
}

function SubjectEventList({ timetable }: { timetable: DailyTimetableDto }) {
    return (
        <div className={style.subjectEventList}>
            {timetable
                .time_blocks
                .entries()
                .map(([_, t], i) => {
                    return <div className={style.subjectEventEntry}>
                        <M3eHeading className={style.subjectEventTitle} key={i} variant="label" size="medium" emphasized>{t.subject}</M3eHeading>
                        <EventList events={t.events} />
                    </div>
                })
            }
        </div>
    )
}

function EventList({ events }: { events: Array<Event> }) {
    return (
        <>
            {events.entries().map(([_, e], i) =>
                <M3eChip key={i} className={style.eventChip} variant="elevated">
                    {e.category.name}
                </M3eChip>)
            }
        </>
    )

}

export default DailyTimetable
