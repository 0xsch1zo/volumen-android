import { M3eAvatar } from "@m3e/react/avatar"
import styles from "./AccountsList.module.css"
import chevronRight from "../assets/chevron_right.svg";
import { Account } from "../../../types";
import CardList from "../../../components/CardList";

function AccountList({ accounts }: { accounts: Array<Account> }) {
    return (
        <div className={styles.accountList}>
            <CardList items={
                accounts.map(account => {
                    if (account.student_name.length == 0)
                        throw Error("empty name string")
                    const monogram = account.student_name.charAt(0)

                    return {
                        id: account.id,
                        elements: {
                            leading: <M3eAvatar>{monogram}</M3eAvatar>,
                            title: account.student_name,
                            subtitle: account.group,
                            trailing: <img className={styles.accountCardChevron} src={chevronRight} />,
                        }
                    }
                })
            }
            />
        </div>
    )
}

export default AccountList;
