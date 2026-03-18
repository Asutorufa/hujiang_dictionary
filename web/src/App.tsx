import { Theme, Tabs } from "@radix-ui/themes";
import { motion } from "framer-motion";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { Toaster } from "sonner";
import { Route, Router, Switch, useLocation } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";
import { useScrollDirection } from "@/hooks/useScrollDirection";
import { lazy, Suspense, useEffect, useState } from "react";
import { BottomNav } from "@/ui/BottomNav";
import { Home as HomeIcon, BookOpen, Layers3, Settings2 } from "lucide-react";
import {
  ROUTE_FLASHCARD,
  ROUTE_HOME,
  ROUTE_CONFIG,
  ROUTE_LOGIN,
  ROUTE_WORDS,
} from "./lib/constants";

// Lazy load pages
const Home = lazy(() => import("./pages/Home"));
const Words = lazy(() => import("./pages/Words"));
const Flashcard = lazy(() => import("./pages/Flashcard"));
const Config = lazy(() => import("./pages/Config"));
const Login = lazy(() => import("./pages/Login"));

const NAV_ROUTE_ORDER = [
  ROUTE_HOME,
  ROUTE_WORDS,
  ROUTE_FLASHCARD,
  ROUTE_CONFIG,
] as const;

const NAV_ROUTE_INDEX = new Map<string, number>(
  NAV_ROUTE_ORDER.map((route, index) => [route, index]),
);

function ThemeWrapper({ children }: { children: React.ReactNode }) {
  const { theme } = useTheme();

  return (
    <Theme
      appearance={theme === "dark" || theme === "light" ? theme : "inherit"}
      accentColor="tomato"
      grayColor="sand"
      radius="large"
      panelBackground="solid"
    >
      {children}
    </Theme>
  );
}

function Main() {
  const [location, setLocation] = useLocation();
  const scrollDirection = useScrollDirection();
  const [pageTransitionDirection, setPageTransitionDirection] = useState(0);

  useEffect(() => {
    const handleUnauthorized = () => {
      setPageTransitionDirection(0);
      setLocation(ROUTE_LOGIN);
    };

    window.addEventListener("unauthorized", handleUnauthorized);
    return () => {
      window.removeEventListener("unauthorized", handleUnauthorized);
    };
  }, [setLocation]);

  const handleNavChange = (nextLocation: string) => {
    if (nextLocation === location) {
      return;
    }

    const currentIndex = NAV_ROUTE_INDEX.get(location);
    const nextIndex = NAV_ROUTE_INDEX.get(nextLocation);

    const direction =
      currentIndex === undefined || nextIndex === undefined
        ? 0
        : nextIndex > currentIndex
          ? 1
          : -1;

    setPageTransitionDirection(direction);
    setLocation(nextLocation);
  };

  return (
    <>
      <Toaster position="bottom-right" richColors />
      {location !== ROUTE_LOGIN && (
        <BottomNav
          value={location}
          onValueChange={handleNavChange}
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
        <Suspense fallback={null}>
          <div className="relative overflow-x-hidden">
            <motion.div
              key={location}
              initial={{
                x:
                  pageTransitionDirection === 0
                    ? 0
                    : pageTransitionDirection > 0
                      ? 44
                      : -44,
              }}
              animate={{ x: 0 }}
              transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
              style={{ width: "100%", willChange: "transform" }}
            >
              <Switch>
                <Route path={ROUTE_HOME} component={Home} />
                <Route path={ROUTE_LOGIN} component={Login} />
                <Route path={ROUTE_WORDS} component={Words} />
                <Route path={ROUTE_FLASHCARD} component={Flashcard} />
                <Route path={ROUTE_CONFIG} component={Config} />
              </Switch>
            </motion.div>
          </div>
        </Suspense>
      </div>
    </>
  );
}

export default function App() {
  return (
    <NextThemesProvider attribute="class" defaultTheme="system">
      <ThemeWrapper>
        <Router hook={useHashLocation}>
          <Main />
        </Router>
      </ThemeWrapper>
    </NextThemesProvider>
  );
}
