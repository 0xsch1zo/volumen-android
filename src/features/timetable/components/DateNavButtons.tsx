import { M3eIconButton } from "@m3e/react/icon-button";
import { M3eButton } from "@m3e/react/button";
import style from "./DateNavButtons.module.css";
import { M3eIcon } from "@m3e/react/icon";
import "@m3e/icons/outlined/chevron_backward";
import "@m3e/icons/outlined/chevron_forward";

function getDayOfMonthString(date: Date): string {
    switch (date.getDate()) {
        case 1:
            return "1st";
        case 2:
            return "2nd";
        case 3:
            return "3rd";
        default:
            return `${date.getDate()}th`;
    }
}


// FIXME: the way I format the date is so much beyond fucked
function DateNavButtons({ current_date }: { current_date: Date }) {
    return <div className={style.container}>
        <M3eIconButton className={style.iconButton} variant="standard" >
            <M3eIcon name="chevron_backward" />
        </M3eIconButton>
        <M3eButton className={style.button} variant="text">
            {`${current_date.toLocaleDateString("en-US", { weekday: "long" })}- ${getDayOfMonthString(current_date)} ${current_date.toLocaleDateString("en-US", { month: "long" })}`}
        </M3eButton>
        <M3eIconButton className={style.iconButton} variant="standard" >
            <M3eIcon name="chevron_forward" />
        </M3eIconButton>
    </div>
}

export default DateNavButtons
