import { Theme, Tabs } from "@radix-ui/themes";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { Toaster } from "sonner";
import { Route, Router, Switch, useLocation } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";
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
import { BottomNav } from "@/ui/BottomNav";
import { Home as HomeIcon, BookOpen, Layers3, Settings2 } from "lucide-react";

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
          <BottomNav
            value={location}
            onValueChange={setLocation}
            hidden={scrollDirection === "down"}
          >
            <Tabs.Trigger value={ROUTE_HOME} className="app-nav-trigger">
              <span className="app-nav-icon">
                <HomeIcon size={18} />
              </span>
              <span className="app-nav-label">Home</span>
            </Tabs.Trigger>
            <Tabs.Trigger value={ROUTE_WORDS} className="app-nav-trigger">
              <span className="app-nav-icon">
                <BookOpen size={18} />
              </span>
              <span className="app-nav-label">Words</span>
            </Tabs.Trigger>
            <Tabs.Trigger value={ROUTE_FLASHCARD} className="app-nav-trigger">
              <span className="app-nav-icon">
                <Layers3 size={18} />
              </span>
              <span className="app-nav-label">Review</span>
            </Tabs.Trigger>
            <Tabs.Trigger value={ROUTE_CONFIG} className="app-nav-trigger">
              <span className="app-nav-icon">
                <Settings2 size={18} />
              </span>
              <span className="app-nav-label">Config</span>
            </Tabs.Trigger>
          </BottomNav>
        )}

        <div
          className={
            location === ROUTE_LOGIN ? undefined : "app-bottom-pad min-h-dvh"
          }
        >
          <Switch>
            <Route path={ROUTE_HOME} component={Home} />
            <Route path={ROUTE_LOGIN} component={Login} />
            <Route path={ROUTE_WORDS} component={Words} />
            <Route path={ROUTE_FLASHCARD} component={Flashcard} />
            <Route path={ROUTE_CONFIG} component={Config} />
          </Switch>
        </div>
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
