import { M3eIconButton } from "@m3e/react/icon-button";
import chevronLeft from "../../../assets/chevrons/chevron_left.svg";
import chevronRight from "../../../assets/chevrons/chevron_right.svg";
import { M3eButton } from "@m3e/react/button";
import style from "./DateNavButtons.module.css";

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
        <M3eIconButton variant="standard">
            <img src={chevronLeft} />
        </M3eIconButton>
        <M3eButton variant="text">
            {`${current_date.toLocaleDateString("en-US", { weekday: "long" })}- ${getDayOfMonthString(current_date)} ${current_date.toLocaleDateString("en-US", { month: "long" })}`}
        </M3eButton>
        <M3eIconButton variant="standard">
            <img src={chevronRight} />
        </M3eIconButton>
    </div>
}

export default DateNavButtons
