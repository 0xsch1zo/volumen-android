import { M3eButton } from "@m3e/react/button";
import { M3eButtonGroup } from "@m3e/react/button-group";
import style from "./PeriodSwitch.module.css";

function PeriodSwitch() {
    return <M3eButtonGroup className={style.periodSwitch} variant="connected">
        <M3eButton variant="tonal" toggle selected>Daily</M3eButton>
        <M3eButton variant="tonal" toggle>Monthly</M3eButton>
    </M3eButtonGroup>
}

export default PeriodSwitch
