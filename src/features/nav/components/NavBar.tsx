import { M3eNavBar, M3eNavItem, M3eNavItemElement } from "@m3e/react/nav-bar";
import { M3eIcon } from "@m3e/react/icon";
import style from "./NavBar.module.css";
import "@m3e/icons/outlined/home";
import "@m3e/icons/outlined/looks_6";
import "@m3e/icons/outlined/calendar_today";
import "@m3e/icons/outlined/mail";
import { useNavigate } from "react-router";
import React, { useEffect, useRef } from "react";

//TODO: initial selection state

function NavItem({ name, path, icon }: { name: string, path: string, icon: React.ReactElement }) {
    let ref = useRef<M3eNavItemElement>(null);
    const navigate = useNavigate()
    useEffect(
        () => ref.current?.addEventListener(
            "change",
            () => navigate(path)
        ),
        [navigate]
    )

    return <M3eNavItem ref={ref}>
        {icon}
        {name}
    </M3eNavItem>

}

function NavBar() {
    // we need to set the first element to slected on page load 
    return (
        <M3eNavBar className={style.navBar}>
            <NavItem
                name="Home"
                path="/home"
                icon={<M3eIcon slot="icon" name="home" />}
            />

            <NavItem
                name="Grades"
                path="/grades"
                icon={<M3eIcon slot="icon" name="looks_6" />}
            />
            <NavItem
                name="Timetable"
                path="/timetable"
                icon={<M3eIcon slot="icon" name="calendar_today" />}
            />
            <NavItem
                name="Messages"
                path="/messages"
                icon={<M3eIcon slot="icon" name="mail" />}
            />
        </M3eNavBar>
    )
}

export default NavBar
