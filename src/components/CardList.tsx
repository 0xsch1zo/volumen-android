import { M3eCard } from "@m3e/react/card";
import style from "./CardList.module.css"
import { M3eHeading } from "@m3e/react/heading";

type ItemProps = {
    leading?: React.ReactNode,
    title: string,
    subtitle: string,
    trailing?: React.ReactNode
    onAction?: () => {},
}

function Item({ leading, title, subtitle, trailing, onAction }: ItemProps) {
    return (
        <M3eCard
            orientation="horizontal"
            className={style.item}
            variant="outlined"
            actionable
            onClick={onAction}
        >
            <div className={style.card}>
                <div className={style.left}>
                    {leading}
                    <div
                        className={style.content}
                    >
                        <M3eHeading
                            variant="title"
                            size="medium">
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
    id: number,
    elements: ItemProps,
}

function CardList({ items }: { items: Array<DistinctItem> }) {
    return (
        <div className={style.list}>
            {items.map(i => {
                return <Item key={i.id} {...i.elements} />
            })}
        </div>
    )
}

export default CardList
