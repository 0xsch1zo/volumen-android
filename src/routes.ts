import {
    index,
    route,
} from "@react-router/dev/routes";

export default [
    //route("login", "./routes/login.tsx"),
    index("./routes/login.tsx"),
    //index("./routes/chooseAccount.tsx"),
    route("choose-account", "./routes/chooseAccount.tsx")
]
