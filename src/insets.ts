import { M3 } from "tauri-plugin-m3";

async function setInsets() {
    const insets = await M3.getInsets();

    console.log(insets)
    if (insets === false) {
        document.documentElement.style.setProperty("--inset-top", "0rem")
        document.documentElement.style.setProperty("--inset-bottom", "0rem")
        document.documentElement.style.setProperty("--inset-right", "0rem")
        document.documentElement.style.setProperty("--inset-left", "0rem")
    } else {
        document.documentElement.style.setProperty("--inset-top", `${insets.adjustedInsetTop}px`)
        document.documentElement.style.setProperty("--inset-bottom", `${insets.adjustedInsetBottom}px`)
        document.documentElement.style.setProperty("--inset-left", `${insets.adjustedInsetLeft}px`)
        document.documentElement.style.setProperty("--inset-right", `${insets.adjustedInsetRight}px`)
    }
}

export { setInsets }
