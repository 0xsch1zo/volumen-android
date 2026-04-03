import {
    Links,
    Meta,
    Outlet,
    Scripts,
    ScrollRestoration,
} from "react-router";
import "./root.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

export const queryClient = new QueryClient()

export function Layout({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en">
            <head>
                <meta charSet="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <style>{`
@media (prefers-color-scheme: light) {
    html {
        background-color: white;
    }
}

@media (prefers-color-scheme: dark) {
    html {
        background-color: black;
    }
}
                `}</style>
                <Meta />
                <Links />
            </head>
            <body>
                <QueryClientProvider client={queryClient}>
                    {children}
                </QueryClientProvider>
                <ScrollRestoration />
                <Scripts />
            </body>
        </html>
    );
}

export default function App() {
    return <Outlet />;
}

