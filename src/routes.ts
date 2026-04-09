import {
    index,
    layout,
    route,
    RouteConfig,
} from "@react-router/dev/routes";

export default [
    layout("./routes/theme.tsx", [
        index("./routes/login.tsx"),
        route("choose-account", "./routes/chooseAccount.tsx"),
        layout("./routes/homeLayout.tsx", [
            route("home", "./routes/home.tsx")
        ]),
    ])
] satisfies RouteConfig
