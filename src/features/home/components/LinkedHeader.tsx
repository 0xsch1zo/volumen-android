import { M3eHeading } from "@m3e/react/heading";
import { M3eIconButton } from "@m3e/react/icon-button";
import { useNavigate } from "react-router";
import style from "./LinkedHeader.module.css";
import "@m3e/icons/outlined/arrow_forward";
import { M3eIcon } from "@m3e/react/icon";

function LinkedHeader({ title, destination }: { title: string, destination: string }) {
    const navigate = useNavigate()
    return (
        <div className={style.headerContainer}>
            <M3eHeading
                variant="title"
                size="medium"
                className={style.header}
            >
                {title}
            </M3eHeading>
            <M3eIconButton onClick={() => navigate(destination)}>
                <M3eIcon name="arrow_forward" />
            </M3eIconButton>
        </div>
    )
}

export default LinkedHeader
