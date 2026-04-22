import { cjk } from "@streamdown/cjk";
import { createMathPlugin } from "@streamdown/math";
import "katex/dist/katex.min.css";
import { FC } from "react";
import rehypeRaw from "rehype-raw";
import { Streamdown } from "streamdown";
import { AudioPlayer } from "./AudioPlayer";

// Using 'any' for the component map type if necessary, but Streamdown should accept it.
// Explicitly casting components if Typescript complains, but standard ReactMarkdown usage is fine.

export const Markdown: FC<{
  children: string | null | undefined;
  className?: string;
}> = ({ children, className }) => {
  if (!children) return null;
  return (
    <Streamdown
      className={className}
      rehypePlugins={[rehypeRaw]}
      plugins={{
        cjk: cjk,
        math: createMathPlugin({ singleDollarTextMath: true }),
      }}
      components={{
        audio: AudioPlayer,
      }}
    >
      {children}
    </Streamdown>
  );
};
