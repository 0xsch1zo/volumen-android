import { useLocation } from "react-router";
import AppBar from "../features/nav/components/AppBar";
import { Account } from "../types";
import NavBar from "../features/nav/components/NavBar";

function HomeLayout({ children }: { children: React.ReactNode }) {
    const state = useLocation().state
    if (state === undefined)
        throw Error("account undefined in home layout(not received through useLocation or incorrect type)")
    const { account }: { account: Account } = state
    return (
        <>
            <AppBar account={account} />
            {children}
            <NavBar />
        </>
    )
}

export default HomeLayout
