import { M3eCard } from "@m3e/react/card";
import { TimeBlock } from "../../../types";
import EventList from "../../../components/EventList";
import { M3eHeading } from "@m3e/react/heading";
import style from "./TimeblockList.module.css";
import { Day } from "../types";

function TimeblockCard({ timeblock }: { timeblock: TimeBlock | null }) {
    let element;
    if (timeblock != null) {
        element = <><div className={style.subjectEventContainer}>
            <M3eHeading variant="title" size="medium" emphasized>{timeblock.subject}</M3eHeading>
            <EventList events={timeblock.events} />
        </div>
            <M3eHeading variant="label" size="large">{timeblock.start} - {timeblock.end}</M3eHeading>
        </>
    } else {
        element = <M3eHeading variant="title" size="medium" emphasized>-</M3eHeading>
    }

    return <M3eCard variant="outlined">{element}</M3eCard>
}

function TimeblockList({ day }: { day: Day }) {
    return <div className={style.list}>
        {[...day.time_blocks.map(timeblock => <TimeblockCard timeblock={timeblock} />)]}
    </div>
}

export default TimeblockList
