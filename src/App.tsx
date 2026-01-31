import { HeroUIProvider, Tab, Tabs, ToastProvider } from "@heroui/react";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { Route, Router, Switch, useLocation } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";
import "@/globals.css";
import { useScrollDirection } from "@/hooks/useScrollDirection";
import Home from "./pages/Home";
import Words from "./pages/Words";
import Flashcard from "./pages/Flashcard";

function Main() {
  const [location, setLocation] = useLocation();
  const scrollDirection = useScrollDirection();
  const { systemTheme } = useTheme();

  return (
    <HeroUIProvider navigate={setLocation}>
      <ToastProvider />
      <NextThemesProvider attribute="class" defaultTheme={systemTheme}>
        <div className={`fixed bottom-15 left-1/2 -translate-x-1/2 z-50 transition-transform duration-300 ${scrollDirection === 'down' ? 'translate-y-32' : 'translate-y-0'}`}>
          <Tabs
            variant="bordered"
            aria-label="Options"
            classNames={{
              tabList: "backdrop-blur-sm shadow-md"
            }}
            selectedKey={location}
          >
            <Tab title="Home" href="/" key="/" />
            <Tab title="Words" href="/docs/words" key="/docs/words" />
            <Tab title="Flashcard" href="/docs/flashcard" key="/docs/flashcard" />
          </Tabs>
        </div>

        <Switch>
            <Route path="/" component={Home} />
            <Route path="/docs/words" component={Words} />
            <Route path="/docs/flashcard" component={Flashcard} />
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
