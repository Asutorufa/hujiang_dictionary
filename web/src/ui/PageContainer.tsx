import { PropsWithChildren } from "react";

export function PageContainer({
  children,
  className,
  size = "4xl",
}: PropsWithChildren<{
  className?: string;
  size?: "3xl" | "4xl" | "7xl";
}>) {
  const max =
    size === "7xl" ? "max-w-7xl" : size === "4xl" ? "max-w-4xl" : "max-w-3xl";
  return (
    <div
      className={`app-page-container ${max} mx-auto px-3 sm:px-4 md:px-5 py-6 ${className || ""}`}
    >
      {children}
    </div>
  );
}
