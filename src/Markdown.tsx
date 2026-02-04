import { FC } from "react";
import rehypeRaw from "rehype-raw";
import remarkGfm from "remark-gfm";
import { Streamdown } from "streamdown";
import { AudioPlayer } from "./AudioPlayer";

// Using 'any' for the component map type if necessary, but Streamdown should accept it.
// Explicitly casting components if Typescript complains, but standard ReactMarkdown usage is fine.

export const Markdown: FC<{ children: string | null | undefined, className?: string }> = ({ children, className }) => {
    if (!children) return null;
    return (
        <Streamdown
            className={className}
            rehypePlugins={[rehypeRaw]}
            remarkPlugins={[remarkGfm]}
            components={{
                audio: AudioPlayer as any // Cast to any to avoid potential strict type mismatches with hast nodes
            }}
        >
            {children}
        </Streamdown>
    );
};
