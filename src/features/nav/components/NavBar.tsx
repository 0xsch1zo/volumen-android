import { M3eNavBar, M3eNavItem, M3eNavItemElement } from "@m3e/react/nav-bar";
import { M3eIcon } from "@m3e/react/icon";
import style from "./NavBar.module.css";
import "@m3e/icons/outlined/home";
import "@m3e/icons/outlined/looks_6";
import "@m3e/icons/outlined/calendar_today";
import "@m3e/icons/outlined/mail";
import { useLocation, useNavigate } from "react-router";
import React, { useEffect, useRef } from "react";


// FIXME: the navbar doesn't reflect the state of the location when navigaating wihout it(scroll for example)

function NavItem({ name, path, icon, initialySelected = false }: { name: string, path: string, icon: React.ReactElement, initialySelected?: boolean }) {
    let ref = useRef<M3eNavItemElement>(null);
    const navigate = useNavigate()
    let selected = useRef<boolean>(false)
    useEffect(
        () => {
            if (selected !== undefined)
                selected.current = initialySelected
            if (ref.current !== null)
                ref.current.selected = initialySelected
        },
        []
    )

    useEffect(
        () => ref.current?.addEventListener(
            "change",
            () => navigate(path)
        ),
        [navigate]
    )

    return <M3eNavItem key={path} ref={ref}>
        {icon}
        {name}
    </M3eNavItem>
}

type Item = {
    name: string,
    key: string,
    path: string,
    icon: React.ReactElement,
    initialySelected?: boolean,
}

function NavBar() {
    let itemMap = new Map<string, Item>([
        ["/home", {
            name: "Home",
            key: "home",
            path: "/home",
            icon: < M3eIcon slot="icon" name="home" />,
        }],
        ["/grades", {
            name: "Grades",
            key: "grades",
            path: "/grades",
            icon: < M3eIcon slot="icon" name="looks_6" />,
        }],
        ["/timetable", {
            name: "Timetable",
            key: "timetable",
            path: "/timetable",
            icon: < M3eIcon slot="icon" name="calendar_today" />,
        }],
        ["/messages", {
            name: "Messages",
            key: "messages",
            path: "/messages",
            icon: < M3eIcon slot="icon" name="mail" />,
        }]
    ])

    const path = useLocation().pathname
    itemMap.get(path)!.initialySelected = true


    console.log(itemMap.values())
    // we need to set the first element to slected on page load 
    return (
        <M3eNavBar className={style.navBar}>
            {[
                itemMap.values().map(
                    (item) => <NavItem
                        name={item.name}
                        key={item.key}
                        path={item.path}
                        icon={item.icon}
                        initialySelected={item.initialySelected}
                    />)
            ]}
        </M3eNavBar>
    )
}

export default NavBar
