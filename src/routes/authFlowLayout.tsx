import { Outlet } from "react-router"
import GeometricBackground from "../components/GeometricBackground"
import type { Route } from "./+types/authFlowLayout"

function AuthFlowLayout({ }: Route.ComponentProps) {
    return <>
        <GeometricBackground />
        <Outlet />
    </>
}

export default AuthFlowLayout
