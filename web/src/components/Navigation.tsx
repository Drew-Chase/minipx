import {Button, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Link, Navbar, NavbarBrand, NavbarContent, NavbarItem} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {useTheme} from "../providers/ThemeProvider";

export default function Navigation()
{
    const {themeConfig, toggleTheme, setColorblindMode} = useTheme();

    return (
        <Navbar
            isBordered
            maxWidth="full"
            classNames={{
                wrapper: "px-6"
            }}
            className="animate-slide-down"
        >
            <NavbarBrand as={Link} href={"/"} className={"dark:text-white text-black font-bold"}>
                Minipx Dashboard
            </NavbarBrand>
            <NavbarContent justify="end">
                <NavbarItem>
                    <Dropdown>
                        <DropdownTrigger>
                            <Button
                                isIconOnly
                                variant="light"
                                radius="full"
                                className="hover:scale-110 transition-transform"
                            >
                                <Icon icon="solar:palette-bold" width="20" height="20"/>
                            </Button>
                        </DropdownTrigger>
                        <DropdownMenu aria-label="Accessibility options">
                            <DropdownItem
                                key="normal"
                                onClick={() => setColorblindMode("none")}
                                startContent={<Icon icon="solar:eye-bold" width="18" height="18"/>}
                            >
                                Normal Vision
                            </DropdownItem>
                            <DropdownItem
                                key="protanopia"
                                onClick={() => setColorblindMode("protanopia")}
                                startContent={<Icon icon="solar:eye-bold" width="18" height="18"/>}
                            >
                                Protanopia
                            </DropdownItem>
                            <DropdownItem
                                key="deuteranopia"
                                onClick={() => setColorblindMode("deuteranopia")}
                                startContent={<Icon icon="solar:eye-bold" width="18" height="18"/>}
                            >
                                Deuteranopia
                            </DropdownItem>
                            <DropdownItem
                                key="tritanopia"
                                onClick={() => setColorblindMode("tritanopia")}
                                startContent={<Icon icon="solar:eye-bold" width="18" height="18"/>}
                            >
                                Tritanopia
                            </DropdownItem>
                        </DropdownMenu>
                    </Dropdown>
                </NavbarItem>

                <NavbarItem>
                    <Button
                        isIconOnly
                        variant="light"
                        radius="full"
                        onPress={toggleTheme}
                        className="hover:scale-110 transition-transform"
                    >
                        {themeConfig.mode === "dark" ? (
                            <Icon icon="solar:sun-bold" width="20" height="20"/>
                        ) : (
                            <Icon icon="solar:moon-bold" width="20" height="20"/>
                        )}
                    </Button>
                </NavbarItem>
            </NavbarContent>
        </Navbar>
    );
}
