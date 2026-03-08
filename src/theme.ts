(async () => {
    let { M3 } = await import("tauri-plugin-m3");

    let colorPalette = await M3.getColors();

    console.log(colorPalette);

    if (colorPalette !== false) {
        for (let [color, value] of Object.entries(colorPalette)) {
            document.documentElement.style.setProperty(`--md-sys-color-${color.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()}`, value)
        }
    } else {
        const DEFAUlT_THEME_SEED = "#4285F4"
        let themedRoot = document.createElement("m3e-theme")
        themedRoot.color = DEFAUlT_THEME_SEED
        themedRoot.motion = "expressive"
        themedRoot.scheme = "auto"
        let newRoot = document.createElement("div")
        newRoot.id = "root"
        themedRoot.appendChild(newRoot)
        document.getElementById("root")?.replaceWith(themedRoot)
    }
})()

