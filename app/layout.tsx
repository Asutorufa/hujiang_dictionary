"use client"

import { HeroUIProvider, Tab, Tabs, ToastProvider } from "@heroui/react";
import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { usePathname, useRouter } from "next/navigation";
import "./globals.css";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const pathname = usePathname();
  const router = useRouter();

  const { systemTheme } = useTheme();

  return (
    <html>
      <body>
        <HeroUIProvider navigate={router.push}>
          <ToastProvider />
          <NextThemesProvider attribute="class" defaultTheme={systemTheme}>
            <div className="fixed bottom-15 left-1/2 -translate-x-1/2 z-50">
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
              </Tabs>
            </div>

            {children}
          </NextThemesProvider>
        </HeroUIProvider>
      </body>
    </html>
  );
}
