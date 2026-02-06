import { Button, Slider } from "@heroui/react";
import { useCallback, useEffect, useRef, useState } from "react";

function PlayIcon({ size = 24, width, height, ...props }: { size?: number, width?: number, height?: number }) {
    return (
        <svg
            aria-hidden="true"
            fill="none"
            focusable="false"
            height={size || height}
            role="presentation"
            viewBox="0 0 24 24"
            width={size || width}
            {...props}
        >
            <path
                d="M5.5 5.5A2.5 2.5 0 0 1 8 3h8a2.5 2.5 0 0 1 2.5 2.5v13a2.5 2.5 0 0 1-2.5 2.5H8A2.5 2.5 0 0 1 5.5 18.5V5.5Z"
                fill="currentColor"
                stroke="currentColor"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                className="hidden" // Placeholder logic, replacing with actual path below
            />
            <path fill="currentColor" d="M7 6v12l10-6z" />
        </svg>
    );
}

function PauseIcon({ size = 24, width, height, ...props }: { size?: number, width?: number, height?: number }) {
    return (
        <svg
            aria-hidden="true"
            fill="none"
            focusable="false"
            height={size || height}
            role="presentation"
            viewBox="0 0 24 24"
            width={size || width}
            {...props}
        >
            <path fill="currentColor" d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
        </svg>
    );
}

const formatTime = (time: number) => {
    if (isNaN(time)) return "00:00";
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
};

export const AudioPlayer = (props: React.AudioHTMLAttributes<HTMLAudioElement>) => {
    const audioRef = useRef<HTMLAudioElement>(null);
    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(0);
    const [isEnded, setIsEnded] = useState(false);

    useEffect(() => {
        const audio = audioRef.current;
        if (!audio) return;

        const onTimeUpdate = () => {
            if (!isEnded) {
                setCurrentTime(audio.currentTime);
            }
        };
        const onLoadedMetadata = () => setDuration(audio.duration);
        const onEnded = () => {
            setIsPlaying(false);
            setIsEnded(true);
            if (audio.duration) setCurrentTime(audio.duration);
        };
        const onPlay = () => {
            setIsPlaying(true);
            setIsEnded(false);
        };
        const onPause = () => setIsPlaying(false);

        audio.addEventListener("timeupdate", onTimeUpdate);
        audio.addEventListener("loadedmetadata", onLoadedMetadata);
        audio.addEventListener("ended", onEnded);
        audio.addEventListener("play", onPlay);
        audio.addEventListener("pause", onPause);

        return () => {
            audio.removeEventListener("timeupdate", onTimeUpdate);
            audio.removeEventListener("loadedmetadata", onLoadedMetadata);
            audio.removeEventListener("ended", onEnded);
            audio.removeEventListener("play", onPlay);
            audio.removeEventListener("pause", onPause);
        };
    }, [isEnded]);

    const togglePlay = useCallback(() => {
        if (audioRef.current) {
            if (isPlaying) {
                audioRef.current.pause();
            } else {
                audioRef.current.play();
            }
        }
    }, [isPlaying]);

    const handleSeek = useCallback((value: number | number[]) => {
        if (audioRef.current) {
            setIsEnded(false);
            const time = Array.isArray(value) ? value[0] : value;
            audioRef.current.currentTime = time;
            setCurrentTime(time);
        }
    }, []);

    // Remove controls from props to avoid default player
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { controls, className, style, ...restProps } = props;

    return (
        <div
            className={`flex items-center gap-3 bg-default-100 dark:bg-default-50 rounded-xl p-3 w-full max-w-md border border-default-200 shadow-sm my-2 ${className || ""}`}
            style={style}
        >
            <audio ref={audioRef} {...restProps} className="hidden" />

            <Button
                isIconOnly
                size="sm"
                variant="flat"
                color="primary"
                radius="full"
                onPress={togglePlay}
                aria-label={isPlaying ? "Pause" : "Play"}
                className="min-w-8 w-8 h-8"
            >
                {isPlaying ? <PauseIcon size={16} /> : <PlayIcon size={16} />}
            </Button>

            <div className="flex-1 flex flex-col justify-center gap-1">
                <Slider
                    size="sm"
                    step={0.01}
                    maxValue={duration || 100}
                    minValue={0}
                    value={isEnded ? duration : currentTime}
                    onChange={handleSeek}
                    aria-label="Audio Progress"
                    color="primary"
                    classNames={{
                        base: "max-w-full w-full gap-0",
                        track: "h-1 cursor-pointer border-x-0",
                        thumb: "w-2 h-2 after:w-2 after:h-2"
                    }}
                />
            </div>

            <div className="text-tiny font-medium text-default-500 tabular-nums min-w-[70px] text-right">
                {formatTime(currentTime)} / {formatTime(duration)}
            </div>
        </div>
    );
};
