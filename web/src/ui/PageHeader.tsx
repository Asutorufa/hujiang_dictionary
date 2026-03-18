import { Flex, Text, Box } from "@radix-ui/themes";
import { PropsWithChildren, ReactNode } from "react";

export function PageHeader({
  title,
  subtitle,
  actions,
  sticky = false,
  density = "default",
  children,
}: PropsWithChildren<{
  title?: string;
  subtitle?: string;
  actions?: ReactNode;
  sticky?: boolean;
  density?: "default" | "compact";
}>) {
  const padY = density === "compact" ? "py-3" : "py-4";
  const childrenMt = density === "compact" ? "mt-2" : "mt-3";
  return (
    <div
      className={[sticky ? "sticky z-40" : "", "mb-4"].join(" ")}
      style={
        sticky ? { top: "calc(env(safe-area-inset-top) + 0.75rem)" } : undefined
      }
    >
      <div className={`app-hero-card app-header-root px-4 sm:px-5 ${padY}`}>
        <div className="app-header-inner">
          <Flex justify="between" align="start" gap="4" wrap="wrap">
            {(title || subtitle) && (
              <Box className="min-w-0 flex-1">
                {title && (
                  <Text
                    as="h1"
                    size={density === "compact" ? "5" : "6"}
                    weight="bold"
                    className="app-header-title block"
                  >
                    {title}
                  </Text>
                )}
                {subtitle && (
                  <Text
                    as="p"
                    size="2"
                    color="gray"
                    className="app-header-subtitle mt-1 block"
                  >
                    {subtitle}
                  </Text>
                )}
              </Box>
            )}
            {actions && <div className="shrink-0">{actions}</div>}
          </Flex>
          {children && <div className={childrenMt}>{children}</div>}
        </div>
      </div>
    </div>
  );
}
