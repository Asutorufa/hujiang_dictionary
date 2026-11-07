import { Theme } from "@radix-ui/themes";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { Toaster } from "sonner";
import { Route, Router, Switch, useLocation } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";
import { lazy, Suspense, useEffect, useState } from "react";
import { AppSidebar } from "@/ui/AppSidebar";
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

function ThemeWrapper({ children }: { children: React.ReactNode }) {
  const { theme } = useTheme();

  return (
    <Theme
      appearance={theme === "dark" || theme === "light" ? theme : "inherit"}
      accentColor="grass"
      grayColor="gray"
      radius="medium"
      panelBackground="solid"
    >
      {children}
    </Theme>
  );
}

function Main() {
  const [location, setLocation] = useLocation();
  const [sidebarCollapsed, setSidebarCollapsed] = useState(
    () => window.localStorage.getItem("dictdeck_sidebar_collapsed") === "true",
  );
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);

  useEffect(() => {
    window.localStorage.setItem(
      "dictdeck_sidebar_collapsed",
      String(sidebarCollapsed),
    );
  }, [sidebarCollapsed]);

  useEffect(() => {
    const handleUnauthorized = () => {
      setLocation(ROUTE_LOGIN);
    };

    window.addEventListener("unauthorized", handleUnauthorized);
    return () => {
      window.removeEventListener("unauthorized", handleUnauthorized);
    };
  }, [setLocation]);

  useEffect(() => {
    setMobileSidebarOpen(false);
  }, [location]);

  const handleNavChange = (nextLocation: string) => {
    if (nextLocation === location) {
      return;
    }

    setLocation(nextLocation);
  };

  const isLogin = location === ROUTE_LOGIN;

  return (
    <>
      <Toaster position="bottom-right" richColors closeButton />
      {!isLogin && (
        <AppSidebar
          value={location}
          onValueChange={handleNavChange}
          collapsed={sidebarCollapsed}
          onToggleCollapsed={() =>
            setSidebarCollapsed((collapsed) => !collapsed)
          }
          mobileOpen={mobileSidebarOpen}
          onToggleMobile={() => setMobileSidebarOpen((open) => !open)}
          onCloseMobile={() => setMobileSidebarOpen(false)}
        />
      )}

      <main
        className={
          isLogin
            ? undefined
            : `app-main${sidebarCollapsed ? " is-sidebar-collapsed" : ""}`
        }
      >
        <Suspense fallback={null}>
          <div key={location} className="app-route">
            <Switch>
              <Route path={ROUTE_HOME} component={Home} />
              <Route path={ROUTE_LOGIN} component={Login} />
              <Route path={ROUTE_WORDS} component={Words} />
              <Route path={ROUTE_FLASHCARD} component={Flashcard} />
              <Route path={ROUTE_CONFIG} component={Config} />
            </Switch>
          </div>
        </Suspense>
      </main>
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
