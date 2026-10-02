import { M3eSearchBar } from "@m3e/react/search";
import { M3eIconButton } from "@m3e/react/icon-button";
import { M3eIcon } from "@m3e/react/icon";
import style from "./MessageSearch.module.css";
import "@m3e/icons/outlined/search";
import "@m3e/icons/outlined/filter_alt";

function MessageSearch() {
    return <M3eSearchBar className={style.search}>
        <M3eIcon slot="leading" name="search" />
        <input slot="input" placeholder="Browse messages" />
        <M3eIconButton slot="trailing">
            <M3eIcon name="filter_alt" />
        </M3eIconButton>
    </M3eSearchBar>
}

export default MessageSearch
