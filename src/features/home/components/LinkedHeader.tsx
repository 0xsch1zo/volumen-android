import { M3eHeading } from "@m3e/react/heading"
import { M3eIconButton } from "@m3e/react/icon-button"
import { useNavigate } from "react-router"
import arrowForwardIcon from "../assets/arrowForward.svg"
import style from "./LinkedHeader.module.css"

function LinkedHeader({ title, destination }: { title: string, destination: string }) {
    const navigate = useNavigate()
    return (
        <div className={style.headerContainer}>
            <M3eHeading
                variant="title"
                size="large"
                className={style.header}
            >
                {title}
            </M3eHeading>
            <M3eIconButton onClick={() => navigate(destination)}>
                <img src={arrowForwardIcon} />
            </M3eIconButton>
        </div>
    )
}

export default LinkedHeader
