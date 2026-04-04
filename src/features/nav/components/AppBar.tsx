import { M3eAppBar } from "@m3e/react/app-bar";
import { M3eAvatar } from "@m3e/react/avatar";
import { M3eHeading } from "@m3e/react/heading";
import { M3eIconButton } from "@m3e/react/icon-button";
import { Account } from "../../../types";

function AppBar({ account }: { account: Account }) {
    if (account.student_name.length == 0)
        throw Error("empty name string")
    const monogram = account.student_name.charAt(0)

    return (
        <M3eAppBar>
            <M3eIconButton slot="leading-icon">
                <img src="../assets/menu.svg" />
            </M3eIconButton>
            <M3eHeading
                size="large"
                variant="title"
                slot="title"
                emphasized
            >
                Volumen
            </M3eHeading>
            <M3eAvatar slot="trailing-icon">
                {monogram}
            </M3eAvatar>

        </M3eAppBar>
    )
}

export default AppBar
