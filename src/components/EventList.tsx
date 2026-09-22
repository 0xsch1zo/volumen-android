import { M3eChip } from "@m3e/react/chips";
import style from "./EventList.module.css"
import { Event } from "../types";

function EventList({ events }: { events: Array<Event> }) {
    return (
        <>
            {[...events.entries().map(([_, e], i) =>
                <M3eChip key={i} className={style.eventChip} variant="elevated">
                    {e.category.name}
                </M3eChip>)]
            }
        </>
    )
}

export default EventList
