import type { Route } from "./+types/chooseAccount"

import { listAccounts } from "../features/chooseAccount/api";
import AccountList from "../features/chooseAccount/components/AccountList";
import { Account } from "../features/chooseAccount/types.ts";
import { M3eHeading } from "@m3e/react/heading";
import GeometricBackground from "../components/GeometricBackground.tsx";
import styles from "./chooseAccount.module.css"
import { useEffect } from "react";
import registerLogoutListener from "../utils/logoutListener.ts";
import { useNavigate } from "react-router";

type LoaderData = {
    accounts: Array<Account>
}

async function clientLoader(): Promise<LoaderData> {
    return { accounts: await listAccounts() }
}

function ChooseAccountPage({
    loaderData
}: Route.ComponentProps) {
    const { accounts } = loaderData
    const navigate = useNavigate()
    useEffect(() => {
        const unlisten = registerLogoutListener(navigate)
        return () => {
            unlisten.then(f => f())
        }
    }, [navigate])

    return (
        <div className={styles.container}>
            <GeometricBackground />
            <div
                className={styles.heading}
            >
                <M3eHeading
                    variant="headline"
                    size="large"
                    emphasized
                >
                    Volumen
                </M3eHeading>
                <M3eHeading
                    variant="title"
                    size="medium"
                >
                    Choose an account
                </M3eHeading>
            </div>
            <AccountList accounts={accounts} />
        </div>
    )
}

export default ChooseAccountPage

export {
    clientLoader,
}
