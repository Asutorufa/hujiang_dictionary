import { Box, Flex, Text } from "@radix-ui/themes";
import { PropsWithChildren, ReactNode } from "react";

export function PageHeader({
  title,
  subtitle,
  actions,
  sticky = false,
  density = "default",
  children,
}: PropsWithChildren<{
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  sticky?: boolean;
  density?: "default" | "compact";
}>) {
  const padY = density === "compact" ? "py-3" : "py-4";
  const titleSize = density === "compact" ? ("4" as const) : ("5" as const);
  const subtitleSize = density === "compact" ? ("1" as const) : ("2" as const);
  const childrenMt = density === "compact" ? "mt-2" : "mt-3";
  return (
    <div
      className={[sticky ? "sticky z-40" : "", "mb-4"].join(" ")}
      style={
        sticky ? { top: "calc(env(safe-area-inset-top) + 0.75rem)" } : undefined
      }
    >
      <div className={`app-nav-surface rounded-2xl px-3 sm:px-4 ${padY}`}>
        <Flex justify="between" align="center" gap="4" wrap="wrap">
          <Box>
            <Text size={titleSize} weight="bold">
              {title}
            </Text>
            {subtitle && (
              <Text size={subtitleSize} color="gray" as="div">
                {subtitle}
              </Text>
            )}
          </Box>
          {actions && <div className="shrink-0">{actions}</div>}
        </Flex>
        {children && <div className={childrenMt}>{children}</div>}
      </div>
    </div>
  );
}
