import { Tabs } from "@radix-ui/themes";
import { PropsWithChildren } from "react";

export function BottomNav({
  value,
  onValueChange,
  hidden,
  children,
}: PropsWithChildren<{
  value: string;
  onValueChange: (value: string) => void;
  hidden?: boolean;
}>) {
  return (
    <div
      className={[
        "fixed left-1/2 -translate-x-1/2 z-50",
        "transition-transform duration-300",
        hidden ? "translate-y-32" : "translate-y-0",
      ].join(" ")}
      // Avoid relying on Tailwind spacing scale for "bottom-*".
      style={{ bottom: "calc(env(safe-area-inset-bottom) + 1rem)" }}
    >
      <div className="app-nav-surface app-bottom-nav px-1.5 py-1.5 rounded-full">
        <Tabs.Root value={value} onValueChange={onValueChange}>
          <Tabs.List size="2" color="blue" aria-label="Navigation">
            {children}
          </Tabs.List>
        </Tabs.Root>
      </div>
    </div>
  );
}
