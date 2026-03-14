import { Button, Flex, Text } from "@radix-ui/themes";
import { PropsWithChildren } from "react";

export function EmptyState({
  title,
  description,
  icon,
  actionLabel,
  onAction,
}: PropsWithChildren<{
  title: string;
  description?: string;
  icon?: React.ReactNode;
  actionLabel?: string;
  onAction?: () => void;
}>) {
  return (
    <Flex
      align="center"
      justify="center"
      direction="column"
      gap="2"
      className="w-full py-12 text-center"
    >
      {icon && (
        <div className="mb-1 rounded-2xl border border-[var(--gray-a6)] bg-[color-mix(in_srgb,var(--color-panel-solid)_88%,transparent)] px-4 py-3 shadow-sm">
          {icon}
        </div>
      )}
      <Text size="4" weight="bold">
        {title}
      </Text>
      {description && (
        <Text size="2" color="gray">
          {description}
        </Text>
      )}
      {actionLabel && onAction && (
        <Button className="mt-2 cursor-pointer" onClick={onAction}>
          {actionLabel}
        </Button>
      )}
    </Flex>
  );
}
