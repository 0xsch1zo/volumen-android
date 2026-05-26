import { setInsets } from "../insets"
import type { Route } from "./+types/insets"
import { Outlet } from "react-router"

async function clientLoader() {
    await setInsets()
}

function InsetsContextLayout({ }: Route.ComponentProps) {
    return (
        <Outlet />
    )
}


export { clientLoader, }
export default InsetsContextLayout
