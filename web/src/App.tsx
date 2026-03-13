import { HeroUIProvider, Tab, Tabs, ToastProvider } from "@heroui/react";
import { ThemeProvider as NextThemesProvider } from "next-themes";
import { Route, Router, Switch, useLocation } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";
import "@/globals.css";
import { useScrollDirection } from "@/hooks/useScrollDirection";
import Home from "./pages/Home";
import Words from "./pages/Words";
import Flashcard from "./pages/Flashcard";
import Login from "./pages/Login";
import {
  ROUTE_FLASHCARD,
  ROUTE_HOME,
  ROUTE_LOGIN,
  ROUTE_WORDS,
} from "./lib/constants";
import { useEffect } from "react";

function Main() {
  const [location, setLocation] = useLocation();
  const scrollDirection = useScrollDirection();

  useEffect(() => {
    const handleUnauthorized = () => {
      setLocation(ROUTE_LOGIN);
    };

    window.addEventListener("unauthorized", handleUnauthorized);
    return () => {
      window.removeEventListener("unauthorized", handleUnauthorized);
    };
  }, [setLocation]);

  return (
    <HeroUIProvider navigate={setLocation}>
      <ToastProvider />
      <NextThemesProvider attribute="class" defaultTheme="system">
        {location !== ROUTE_LOGIN && (
          <div
            className={`fixed bottom-15 left-1/2 -translate-x-1/2 z-50 transition-transform duration-300 ${scrollDirection === "down" ? "translate-y-32" : "translate-y-0"}`}
          >
            <Tabs
              variant="bordered"
              aria-label="Options"
              classNames={{
                tabList: "backdrop-blur-sm shadow-md",
              }}
              selectedKey={location}
            >
              <Tab title="Home" href={ROUTE_HOME} key={ROUTE_HOME} />
              <Tab title="Words" href={ROUTE_WORDS} key={ROUTE_WORDS} />
              <Tab
                title="Flashcard"
                href={ROUTE_FLASHCARD}
                key={ROUTE_FLASHCARD}
              />
            </Tabs>
          </div>
        )}

        <Switch>
          <Route path={ROUTE_HOME} component={Home} />
          <Route path={ROUTE_LOGIN} component={Login} />
          <Route path={ROUTE_WORDS} component={Words} />
          <Route path={ROUTE_FLASHCARD} component={Flashcard} />
        </Switch>
      </NextThemesProvider>
    </HeroUIProvider>
  );
}

export default function App() {
  return (
    <Router hook={useHashLocation}>
      <Main />
    </Router>
  );
}
