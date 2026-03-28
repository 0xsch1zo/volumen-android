import { Outlet } from "react-router"
import initTheme from "../theme"
import style from "./theme.module.css"

async function clientLoader() {
    await initTheme()
}

function HydrateFallback() {
    return (
        <div className={style.fallback} />
    )
}

function LoadedBackground() {
    return (
        <div className={style.loadedBackground} >
            <Outlet />
        </div>
    )
}

export default LoadedBackground
export {
    clientLoader, HydrateFallback
}
