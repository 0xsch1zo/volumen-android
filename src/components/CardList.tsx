import { M3eCard } from "@m3e/react/card";
import style from "./CardList.module.css"
import { M3eHeading } from "@m3e/react/heading";

type ItemProps = {
    leading?: React.ReactNode,
    header?: string,
    title: string,
    subtitle: string,
    trailing?: React.ReactNode
    onAction?: () => void,
}

function Item({ leading, header, title, subtitle, trailing, onAction }: ItemProps) {
    let leading_contained = leading !== undefined ? <div className={style.leading}>
        {leading}
    </div> : undefined;
    return (
        <M3eCard
            orientation="horizontal"
            variant="outlined"
            actionable
            onClick={() => {
                if (onAction !== undefined) {
                    onAction()
                }
            }
            }
        >
            <div className={style.card}>
                {leading_contained}
                <div className={style.left}>
                    <div
                        className={style.content}
                    >
                        <M3eHeading
                            variant="title"
                            size="small">
                            {header}
                        </M3eHeading>
                        <M3eHeading
                            variant="title"
                            size="small">
                            {title}
                        </M3eHeading>
                        <p className={style.subtitle}>{subtitle}</p>
                    </div>
                </div>
                {trailing}
            </div>
        </M3eCard>
    )
}

interface DistinctItem {
    key: number,
    props: ItemProps,
}

function CardList({ items }: { items: Array<DistinctItem> }) {
    return (
        <div className={style.list}>
            {items.map(i => {
                return <Item key={i.key} {...i.props} />
            })}
        </div>
    )
}

export default CardList
