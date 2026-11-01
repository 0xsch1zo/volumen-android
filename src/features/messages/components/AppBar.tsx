import { M3eAppBar } from "@m3e/react/app-bar";
import { M3eHeading } from "@m3e/react/heading";
import { M3eIconButton } from "@m3e/react/icon-button";
import style from "./AppBar.module.css"
import "@m3e/icons/outlined/menu";
import { M3eIcon } from "@m3e/react/icon";

function AppBar() {
    return (
        <M3eAppBar className={style.appBar}>
            <M3eIconButton
                slot="leading-icon"
            >
                <M3eIcon name="menu" />
            </M3eIconButton>
            <M3eHeading
                size="large"
                variant="title"
                slot="title"
                emphasized
            >
                Volumen
            </M3eHeading>
        </M3eAppBar>
    )
}

export default AppBar
