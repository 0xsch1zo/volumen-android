import style from "./GeometricBackground.module.css"

function GeometricBackground() {
    return (
        <div className={style.container}>
            <div className={style.sixSidedCookie}></div>
            <div className={style.nineSidedCookie}></div>
            <div className={style.circle}></div>
            <div className={style.pentagon}></div>
        </div>
    );
}

export default GeometricBackground
