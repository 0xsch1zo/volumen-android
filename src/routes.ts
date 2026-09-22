import {
    index,
    layout,
    route,
    RouteConfig,
} from "@react-router/dev/routes";

export default [
    layout("./routes/theme.tsx", [
        layout("./routes/insets.tsx", [
            index("./routes/login.tsx"),
            route("choose-account", "./routes/chooseAccount.tsx"),
            layout("./routes/appLayout.tsx", [
                route("home", "./routes/home.tsx"),
                route("grades", "./routes/grades.tsx"),
                route("timetable", "./routes/timetable.tsx"),
            ])
        ]),
    ])
] satisfies RouteConfig
