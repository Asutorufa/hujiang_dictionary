"use client"

import { HeroUIProvider, Tab, Tabs } from "@heroui/react";
import { usePathname, useRouter } from "next/navigation";
import "./globals.css";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const pathname = usePathname();
  const router = useRouter();

  return (
    <html lang="en">
      <body>
        <HeroUIProvider navigate={router.push}>
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
        </HeroUIProvider>
      </body>
    </html>
  );
}
