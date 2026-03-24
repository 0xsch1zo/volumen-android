import style from "./PageBackground.module.css"

function PageBackground() {
    return (
        <div className={style.container}>
            <div className={style.sixSidedCookie}></div>
            <div className={style.nineSidedCookies}></div>
            <div className={style.circle}></div>
            <div className={style.pentagon}></div>
        </div>
    );
}

export default PageBackground;
