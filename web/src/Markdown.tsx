import { FC, lazy, Suspense } from "react";

// Streamdown + KaTeX + Shiki are by far the largest dependencies. Load them only
// when a page actually renders Markdown instead of on the initial page load.
const MarkdownRenderer = lazy(() => import("./MarkdownRenderer"));

export const Markdown: FC<{
  children: string | null | undefined;
  className?: string;
}> = ({ children, className }) => {
  if (!children) return null;
  return (
    <Suspense
      fallback={
        <div className={className} style={{ whiteSpace: "pre-wrap" }}>
          {children}
        </div>
      }
    >
      <MarkdownRenderer className={className}>{children}</MarkdownRenderer>
    </Suspense>
  );
};
