import { Button } from "@repo/ui/components/button";
import { ModeToggle } from "@repo/ui/components/mode-toggle";

import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@repo/ui/components/navigation-menu";
import { Separator } from "@repo/ui/components/separator";
import {
  Sheet,
  SheetContent,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@repo/ui/components/sheet";
import { Link, type LinkProps } from "@tanstack/react-router";
import { Menu } from "lucide-react";
import { useState } from "react";
import Logo from "../../../../app-icon.svg";

type RouteDef = Pick<LinkProps, "to"> & {
  label: string;
};

const routeList: RouteDef[] = [
  {
    label: "New Analysis",
    to: "/",
  },
  {
    label: "Settings",
    to: "/settings",
  },
  {
    label: "Analysis",
    to: "/analysis",
  },
];

export const NavBar = () => {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <header className="sticky z-40 mx-auto flex items-center justify-between border bg-card/25 p-2 ">
      {/* Placeholder for Logo */}
      <div className="flex items-center md:hidden">
        <Sheet open={isOpen} onOpenChange={setIsOpen}>
          <SheetTrigger asChild>
            <Menu
              onClick={() => setIsOpen(!isOpen)}
              className="cursor-pointer lg:hidden"
            />
          </SheetTrigger>

          <SheetContent
            side="left"
            className="flex flex-col justify-between rounded-tr-2xl rounded-br-2xl border-secondary bg-card"
          >
            <div>
              <SheetHeader className="mb-4 ml-4">
                <SheetTitle className="flex items-center">
                  <a href="/" className="flex items-center">
                    {/* biome-ignore lint/nursery/noImgElement: Importing svg as image */}
                    <img alt="logo" src={Logo} className="mr-2 h-9 w-9" />
                    Serpeo
                  </a>
                </SheetTitle>
              </SheetHeader>

              <div className="flex flex-col gap-2">
                {routeList.map(({ label, ...rest }) => (
                  <Button
                    key={label}
                    onClick={() => setIsOpen(false)}
                    asChild
                    variant="ghost"
                    className="justify-start text-base"
                  >
                    <Link {...rest} viewTransition>
                      {label}
                    </Link>
                  </Button>
                ))}
              </div>
            </div>

            <SheetFooter className="flex-col items-start justify-start sm:flex-col">
              <Separator className="mb-2" />

              <ModeToggle />
            </SheetFooter>
          </SheetContent>
        </Sheet>
      </div>

      {/* <!-- Desktop --> */}
      <NavigationMenu className="mx-auto hidden md:block">
        <NavigationMenuList>
          <NavigationMenuItem className="flex flex-row">
            {routeList.map(({ label, ...rest }) => (
              <NavigationMenuLink key={label} asChild>
                <Link {...rest} viewTransition className="px-2 text-base">
                  {label}
                </Link>
              </NavigationMenuLink>
            ))}
          </NavigationMenuItem>
        </NavigationMenuList>
      </NavigationMenu>

      <div className="hidden md:flex">
        <ModeToggle />
      </div>
    </header>
  );
};
