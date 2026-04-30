import { cjk } from "@streamdown/cjk";
import { createMathPlugin } from "@streamdown/math";
import "katex/dist/katex.min.css";
import { FC } from "react";
import { Streamdown } from "streamdown";
import { AudioPlayer } from "./AudioPlayer";

export const Markdown: FC<{
  children: string | null | undefined;
  className?: string;
}> = ({ children, className }) => {
  if (!children) return null;
  return (
    <Streamdown
      className={className}
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
