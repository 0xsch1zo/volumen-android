import { M3eAvatar } from "@m3e/react/avatar"
import styles from "./AccountsList.module.css"
import { Account } from "../../../types";
import CardList from "../../../components/CardList";
import { selectAccount } from "../api";
import { useNavigate } from "react-router";
import "@m3e/icons/outlined/chevron_forward";
import { M3eIcon } from "@m3e/react/icon";

function AccountList({ accounts }: { accounts: Array<Account> }) {
    const navigate = useNavigate()
    return (
        <div className={styles.accountList}>
            <CardList
                items={
                    accounts.map(account => {
                        if (account.student_name.length == 0)
                            throw Error("empty name string")
                        const monogram = account.student_name.charAt(0)

                        return {
                            key: account.id,
                            props: {
                                leading: <M3eAvatar>{monogram}</M3eAvatar>,
                                title: account.student_name,
                                subtitle: account.group,
                                trailing: <M3eIcon name="chevron_forward" className={styles.accountCardChevron} />,
                                onAction: async () => {
                                    await selectAccount(account)
                                    navigate("/home", {
                                        state: { account },
                                    })
                                }
                            },
                        }
                    })
                }
            />
        </div>
    )
}

export default AccountList;
