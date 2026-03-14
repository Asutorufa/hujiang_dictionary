import { Theme, Tabs } from "@radix-ui/themes";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { Toaster } from "sonner";
import { Route, Router, Switch, useLocation } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";
import "@/globals.css";
import { useScrollDirection } from "@/hooks/useScrollDirection";
import Home from "./pages/Home";
import Words from "./pages/Words";
import Flashcard from "./pages/Flashcard";
import Config from "./pages/Config";
import Login from "./pages/Login";
import {
  ROUTE_FLASHCARD,
  ROUTE_HOME,
  ROUTE_CONFIG,
  ROUTE_LOGIN,
  ROUTE_WORDS,
} from "./lib/constants";
import { useEffect } from "react";

function ThemeWrapper({ children }: { children: React.ReactNode }) {
  const { theme } = useTheme();

  return (
    <Theme
      appearance={theme === "dark" || theme === "light" ? theme : "inherit"}
      accentColor="blue"
      grayColor="slate"
      radius="large"
    >
      {children}
    </Theme>
  );
}

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
    <NextThemesProvider attribute="class" defaultTheme="system">
      <ThemeWrapper>
        <Toaster position="bottom-right" richColors />
        {location !== ROUTE_LOGIN && (
          <div
            className={`fixed bottom-15 left-1/2 -translate-x-1/2 z-50 transition-transform duration-300 ${scrollDirection === "down" ? "translate-y-32" : "translate-y-0"}`}
          >
            <div className="bg-background/80 backdrop-blur-sm shadow-md rounded-full px-2 border border-default-200">
              <Tabs.Root value={location} onValueChange={setLocation}>
                <Tabs.List size="2" color="blue" aria-label="Navigation">
                  <Tabs.Trigger value={ROUTE_HOME}>Home</Tabs.Trigger>
                  <Tabs.Trigger value={ROUTE_WORDS}>Words</Tabs.Trigger>
                  <Tabs.Trigger value={ROUTE_FLASHCARD}>Flashcard</Tabs.Trigger>
                  <Tabs.Trigger value={ROUTE_CONFIG}>Config</Tabs.Trigger>
                </Tabs.List>
              </Tabs.Root>
            </div>
          </div>
        )}

        <Switch>
          <Route path={ROUTE_HOME} component={Home} />
          <Route path={ROUTE_LOGIN} component={Login} />
          <Route path={ROUTE_WORDS} component={Words} />
          <Route path={ROUTE_FLASHCARD} component={Flashcard} />
          <Route path={ROUTE_CONFIG} component={Config} />
        </Switch>
      </ThemeWrapper>
    </NextThemesProvider>
  );
}

export default function App() {
  return (
    <Router hook={useHashLocation}>
      <Main />
    </Router>
  );
}
