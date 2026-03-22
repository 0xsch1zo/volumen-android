import { M3 } from "tauri-plugin-m3";
import {
    themeFromSourceColor,
    argbFromHex,
    applyTheme,
} from "@material/material-color-utilities";

async function initTheme() {
    const colorPalette = await M3.getColors();

    const failure = Object.entries(colorPalette).length == 1 && 'error' in (colorPalette as unknown as object);
    if (colorPalette !== false && !failure) {
        for (let [color, value] of Object.entries(colorPalette)) {
            document.documentElement.style.setProperty(`--md-sys-color-${color.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase()}`, value as string)
        }
    } else {
        const theme = themeFromSourceColor(argbFromHex("#4285F4"));
        const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        applyTheme(theme, { target: document.body, dark: isDark })
    }
}

export default initTheme;
