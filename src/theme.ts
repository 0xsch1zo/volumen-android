import { M3 } from "tauri-plugin-m3";
import {
    themeFromSourceColor,
    argbFromHex,
    applyTheme,
    hexFromArgb,
} from "@material/material-color-utilities";
import { ColorScheme } from "tauri-plugin-m3";

async function initTheme() {
    const colorPalette = await M3.getColors();

    const failure = Object.entries(colorPalette).length == 1 && 'error' in (colorPalette as unknown as object);
    if (colorPalette !== false && !failure) {
        applyDynamicTheme(colorPalette)
    } else {
        applyDefaultTheme()
    }
}


function applyDynamicTheme(colorPalette: ColorScheme) {
    for (let [color, value] of Object.entries(colorPalette)) {
        if (value === undefined) {
            applyDefaultTheme()
            throw Error("undefined color value, using default theme")
        }
        document.documentElement.style.setProperty(`--md-sys-color-${color.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()}`, value as string)
    }
    document.documentElement.style.setProperty("background-color", colorPalette.background as string)
}

function applyDefaultTheme() {
    const theme = themeFromSourceColor(argbFromHex("#4285F4"));
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (isDark)
        document.documentElement.style.setProperty("background-color", hexFromArgb(theme.schemes.dark.background))
    else
        document.documentElement.style.setProperty("background-color", hexFromArgb(theme.schemes.light.background))

    applyTheme(theme, { target: document.documentElement, dark: isDark })
}

export default initTheme;
