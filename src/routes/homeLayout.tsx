import type { Route } from "./+types/homeLayout"
import { Outlet } from "react-router";
import AppBar from "../features/nav/components/AppBar";
import NavBar from "../features/nav/components/NavBar";
import { currentAccount } from "../utils";
import { Account } from "../types";
import { queryClient } from "../root";

async function clientLoader(): Promise<Account> {
    return await queryClient.ensureQueryData({
        queryKey: ["currentAccount"],
        queryFn: currentAccount,
    })
}

function HomeLayout({ loaderData }: Route.ComponentProps) {
    const account = loaderData
    return (
        <>
            <AppBar account={account} />
            <Outlet />
            <NavBar />
        </>
    )
}

export default HomeLayout
export { clientLoader }
