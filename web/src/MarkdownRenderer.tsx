import { cjk } from "@streamdown/cjk";
import { createMathPlugin } from "@streamdown/math";
import "katex/dist/katex.min.css";
import { Streamdown } from "streamdown";
import { AudioPlayer } from "./AudioPlayer";

// Hoisted so Streamdown receives stable references and does not rebuild its
// processor pipeline on every streamed chunk.
const plugins = {
  cjk,
  math: createMathPlugin({ singleDollarTextMath: true }),
};

const components = {
  audio: AudioPlayer,
};

export default function MarkdownRenderer({
  children,
  className,
}: {
  children: string;
  className?: string;
}) {
  return (
    <Streamdown className={className} plugins={plugins} components={components}>
      {children}
    </Streamdown>
  );
}
