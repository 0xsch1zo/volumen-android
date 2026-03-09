(async () => {
    let { M3 } = await import("tauri-plugin-m3");

    let colorPalette = await M3.getColors();

    let failure = Object.entries(colorPalette).length == 1 && 'error' in (colorPalette as unknown as object);
    if (colorPalette !== false && !failure) {
        for (let [color, value] of Object.entries(colorPalette)) {
            document.documentElement.style.setProperty(`--md-sys-color-${color.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()}`, value as string)
        }
    } else {
        let {
            themeFromSourceColor,
            argbFromHex,
            applyTheme,
        } = await import("@material/material-color-utilities");

        const theme = themeFromSourceColor(argbFromHex("#4285F4"));
        const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        applyTheme(theme, { target: document.body, dark: isDark })
    }
})()
