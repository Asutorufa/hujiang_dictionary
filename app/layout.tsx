"use client"

import { HeroUIProvider, Tab, Tabs, ToastProvider } from "@heroui/react";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { usePathname, useRouter } from "next/navigation";
import "./globals.css";
import { useScrollDirection } from "@/app/hooks/useScrollDirection";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const pathname = usePathname();
  const router = useRouter();
  const scrollDirection = useScrollDirection();

  const { systemTheme } = useTheme();

  return (
    <html>
      <body>
        <HeroUIProvider navigate={router.push}>
          <ToastProvider />
          <NextThemesProvider attribute="class" defaultTheme={systemTheme}>
            <div className={`fixed bottom-15 left-1/2 -translate-x-1/2 z-50 transition-transform duration-300 ${scrollDirection === 'down' ? 'translate-y-32' : 'translate-y-0'}`}>
              <Tabs
                variant="bordered"
                aria-label="Options"
                classNames={{
                  tabList: "backdrop-blur-sm shadow-md"
                }}
                selectedKey={pathname}
              >
                <Tab title="Home" href="/" key="/" />
                <Tab title="Words" href="/docs/words" key="/docs/words" />
                <Tab title="Flashcard" href="/docs/flashcard" key="/docs/flashcard" />
              </Tabs>
            </div>

            {children}
          </NextThemesProvider>
        </HeroUIProvider>
      </body>
    </html>
  );
}
