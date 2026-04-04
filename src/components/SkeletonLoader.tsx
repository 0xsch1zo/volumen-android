import style from "./SkeletonLoader.module.css"

function SkeletonLoader({ width, height }: { width: string, height: string }) {
    return (<div
        className={style.skeleton}
        style={{
            width: width,
            height: height,
        }}
    />
    )
}

export default SkeletonLoader
