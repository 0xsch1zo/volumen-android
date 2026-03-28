import { M3eAvatar } from "@m3e/react/avatar"
import { M3eCard } from "@m3e/react/card"
import { M3eHeading } from "@m3e/react/heading"
import { Account } from "../types"
import styles from "./AccountsList.module.css"
import chevronRight from "../assets/chevron_right.svg";

function AccountItem({ account }: { account: Account }) {
    if (account.student_name.length == 0)
        throw Error("empty name string")
    const monogram = account.student_name.charAt(0)

    return (
        <M3eCard
            orientation="horizontal"
            className={styles.accountItem}
            variant="outlined"
            actionable
        >
            <div className={styles.accountCard}>
                <div className={styles.accountCardLeft}>
                    <M3eAvatar>{monogram}</M3eAvatar>
                    <div
                        className={styles.accountCardContent}
                    >
                        <M3eHeading
                            variant="title"
                            size="medium">
                            {account.student_name}
                        </M3eHeading>
                        <p className={styles.accountCardGroupText}>{account.group}</p>
                    </div>
                </div>
                <img className={styles.accountCardChevron} src={chevronRight} />
            </div>
        </M3eCard>
    )
}

function AccountList({ accounts }: { accounts: Array<Account> }) {
    return (
        <div className={styles.accountList}>
            {accounts.map(account => (
                <AccountItem account={account} key={account.id} />
            ))}
        </div>
    )
}

export default AccountList;
