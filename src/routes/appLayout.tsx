import type { Route } from "./+types/appLayout";
import { Outlet } from "react-router";
import AppBar from "../features/nav/components/AppBar";
import NavBar from "../features/nav/components/NavBar";
import { currentAccount } from "../utils";
import { Account } from "../types";
import { queryClient } from "../root";
import style from "./appLayout.module.css";

async function clientLoader(): Promise<Account> {
    return await queryClient.ensureQueryData({
        queryKey: ["currentAccount"],
        queryFn: currentAccount,
    })
}

function AppLayout({ loaderData }: Route.ComponentProps) {
    const account = loaderData
    return (
        <div className={style.screen}>
            <AppBar account={account} />
            <div className={style.outletContainer}>
                <Outlet />
            </div>
            <NavBar />
            <div className={style.barInsetFiller} />
        </div>
    )
}

export default AppLayout
export { clientLoader }
