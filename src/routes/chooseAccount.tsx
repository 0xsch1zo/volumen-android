import type { Route } from "./+types/chooseAccount"

import { listAccounts } from "../features/chooseAccount/api";
import AccountList from "../features/chooseAccount/components/AccountList";
import { Account } from "../types.ts";
import { M3eHeading } from "@m3e/react/heading";
import GeometricBackground from "../components/GeometricBackground.tsx";
import styles from "./chooseAccount.module.css"
import { queryClient } from "../root.tsx";

type LoaderData = {
    accounts: Array<Account>
}

async function clientLoader(): Promise<LoaderData> {
    const accounts = await queryClient.fetchQuery({
        queryKey: ["listAccounts"],
        queryFn: listAccounts,
    })
    return { accounts }
}

function ChooseAccountPage({
    loaderData
}: Route.ComponentProps) {
    const { accounts } = loaderData
    return (
        <div className={styles.container}>
            <GeometricBackground />
            <div
                className={styles.heading}
            >
                <M3eHeading
                    variant="headline"
                    size="medium"
                    emphasized
                >
                    Volumen
                </M3eHeading>
                <M3eHeading
                    variant="title"
                    size="small"
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
