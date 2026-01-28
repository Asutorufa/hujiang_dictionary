"use client";

import { BookIcon, changePriority, countWord, FilterIcon, getPriorityColor, incrementRemindCount, ListWordResponse, queryWord, Spoiler } from "@/app/components";
import { addToast, Button, Card, CardBody, CardHeader, Divider, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Spinner, Tab, Tabs } from "@heroui/react";
import { AnimatePresence, motion, PanInfo, useAnimation, useMotionValue, useTransform } from "framer-motion";
import { useCallback, useEffect, useRef, useState } from "react";
import rehypeRaw from "rehype-raw";
import remarkGfm from "remark-gfm";
import { Streamdown } from "streamdown";
import { useLocalStorage } from "usehooks-ts";

// Helper for swipe icons
const CheckIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10s10-4.48 10-10S17.52 2 12 2m-2 15l-5-5l1.41-1.41L10 14.17l7.59-7.59L19 8z" /></svg>
);

const CrossIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.47 2 2 6.47 2 12s4.47 10 10 10s10-4.47 10-10S17.53 2 12 2m5 13.59L15.59 17L12 13.41L8.41 17L7 15.59L10.59 12L7 8.41L8.41 7L12 10.59L15.59 7L17 8.41L13.41 12z" /></svg>
);

const LeftArrowIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z" /></svg>
);

export default function Flashcard() {
    const [page, setPage] = useLocalStorage<number>("flashcard_page_v2", 1);
    const [total, setTotal] = useLocalStorage<number>("total_page", 100);
    const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");
    const [grammar, setGrammar] = useLocalStorage("grammar", false);

    const [wordsMap, setWordsMap] = useState<Map<number, ListWordResponse>>(new Map());
    const [loading, setLoading] = useState(false);
    const loadedChunksRef = useRef<Set<number>>(new Set());
    const longPressTimer = useRef<NodeJS.Timeout | null>(null);

    const CHUNK_SIZE = 10;

    const fetchChunksIfNeeded = useCallback(async (targetIndex: number) => {
        const targetPage = Math.ceil(targetIndex / CHUNK_SIZE);

        const neededChunks: number[] = [];

        // Check if we loaded this chunk
        if (!loadedChunksRef.current.has(targetPage)) {
             neededChunks.push(targetPage);
        }

        // Check next chunk
        if (targetIndex % CHUNK_SIZE > 5) { // If past half-way
             // We can just check the chunk index
             if (!loadedChunksRef.current.has(targetPage + 1)) {
                 neededChunks.push(targetPage + 1);
             }
        }

        if (neededChunks.length > 0) {
            setLoading(true);
            // Mark as loading/loaded to prevent duplicate requests
            neededChunks.forEach(c => loadedChunksRef.current.add(c));

            await Promise.all(neededChunks.map(p =>
                new Promise<void>(resolve => {
                    queryWord(p, CHUNK_SIZE, orderBy, grammar, (data) => {
                        if (data) {
                            setWordsMap(prev => {
                                const newMap = new Map(prev);
                                data.forEach((w, i) => {
                                    // Calculate absolute index
                                    // (p-1)*CHUNK + 1 + i
                                    const absIndex = (p - 1) * CHUNK_SIZE + 1 + i;
                                    newMap.set(absIndex, w);
                                });
                                return newMap;
                            });
                        }
                        resolve();
                    });
                })
            ));
            setLoading(false);
        }
    }, [orderBy, grammar, CHUNK_SIZE]);

    // Initial load and on change
    useEffect(() => {
        // Reset map on filter change
        setWordsMap(new Map());
        loadedChunksRef.current.clear();
        fetchChunksIfNeeded(page);
    }, [orderBy, grammar, fetchChunksIfNeeded, page]);

    useEffect(() => {
        fetchChunksIfNeeded(page);

        // Count update
        countWord(grammar, (size) => {
            if (size) setTotal(size);
        });
    }, [page, grammar, orderBy, fetchChunksIfNeeded, setTotal]);

    const currentWord = wordsMap.get(page);

    // Animation controls
    const controls = useAnimation();
    const x = useMotionValue(0);
    const rotate = useTransform(x, [-200, 200], [-10, 10]);
    const opacityRight = useTransform(x, [50, 150], [0, 1]);
    const opacityLeft = useTransform(x, [-150, -50], [1, 0]);

    const handleSwipe = async (action: 'know' | 'skip') => {
        if (action === 'know') {
            await controls.start({ x: 500, opacity: 0 });
            if (currentWord) {
                await incrementRemindCount(currentWord.word, () => {});
            }
        } else { // 'skip'
            await controls.start({ x: -500, opacity: 0 });
        }
        // Move next
        setPage(p => Math.min(p + 1, total));
    };

    const handleDragEnd = async (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => {
        const threshold = 100;
        if (info.offset.x > threshold) {
            await handleSwipe('know');
        } else if (info.offset.x < -threshold) {
            await handleSwipe('skip');
        } else {
            controls.start({ x: 0, rotate: 0, scale: 1 });
        }
    };

    const handleLongPress = () => {
        if (currentWord) {
            navigator.clipboard.writeText(currentWord.word).then(() => {
                addToast({ title: "Copied!", color: "success" });
            });
        }
    };

    return (
        <div className="flex flex-col h-[calc(100vh-80px)] overflow-hidden items-center relative p-4">
            {/* Header / Filter Bar */}
            <div className="flex gap-2 mb-4 z-10 w-full justify-center">
                <Dropdown>
                    <DropdownTrigger>
                        <Button isDisabled={loading && wordsMap.size === 0} variant="bordered" className="shadow-md backdrop-blur-sm capitalize">
                            <FilterIcon /> {orderBy.replace("_", " ")}
                        </Button>
                    </DropdownTrigger>
                    <DropdownMenu
                        selectionMode="single"
                        selectedKeys={[orderBy]}
                        onSelectionChange={(e) => {
                            const val = e.currentKey ? String(e.currentKey) : "word";
                            setOrderBy(val);
                            setPage(1);
                        }}
                    >
                        <DropdownItem key="word">Word</DropdownItem>
                        <DropdownItem key="word desc">Word DESC</DropdownItem>
                        <DropdownItem key="priority">Priority</DropdownItem>
                        <DropdownItem key="priority desc">Priority DESC</DropdownItem>
                        <DropdownItem key="add_time">Add Time</DropdownItem>
                        <DropdownItem key="add_time desc">Add Time DESC</DropdownItem>
                        <DropdownItem key="update_time">Update Time</DropdownItem>
                        <DropdownItem key="update_time desc">Update Time DESC</DropdownItem>
                        <DropdownItem key="reminder_time">Reminder</DropdownItem>
                        <DropdownItem key="reminder_time desc">Reminder DESC</DropdownItem>
                        <DropdownItem key="anki_count">Count</DropdownItem>
                        <DropdownItem key="anki_count desc">Count DESC</DropdownItem>
                    </DropdownMenu>
                </Dropdown>

                <Dropdown>
                    <DropdownTrigger>
                        <Button isDisabled={loading && wordsMap.size === 0} variant="bordered" className="shadow-md backdrop-blur-sm capitalize">
                            <BookIcon /> {grammar ? "Grammar" : "Word"}
                        </Button>
                    </DropdownTrigger>
                    <DropdownMenu
                        selectionMode="single"
                        selectedKeys={[grammar ? "grammar" : "word"]}
                        onSelectionChange={(e) => {
                            setGrammar(e.currentKey === "grammar");
                            setPage(1);
                        }}
                    >
                        <DropdownItem key="word">Word</DropdownItem>
                        <DropdownItem key="grammar">Grammar</DropdownItem>
                    </DropdownMenu>
                </Dropdown>

                <div className="flex items-center ml-2 px-3 py-1 bg-default-100 rounded-lg text-small">
                    {page} / {total}
                </div>
            </div>

            {/* Main Content Area */}
            <div className="flex-1 w-full max-w-md flex items-center justify-center relative">
                {loading && wordsMap.size === 0 && <Spinner size="lg" />}

                {!loading && wordsMap.size === 0 && (
                    <div className="text-center text-default-500">
                        <p>No words found.</p>
                        <Button className="mt-4" onPress={() => setPage(1)}>Reset to start</Button>
                    </div>
                )}

                <AnimatePresence mode="wait">
                    {currentWord && (
                        <motion.div
                            key={page} // Key change triggers mount/unmount animation
                            initial={{ opacity: 0, y: 50, scale: 0.9 }}
                            animate={{ opacity: 1, y: 0, scale: 1 }}
                            exit={{ opacity: 0, x: 0 }} // Exit handled by manual controls for swipe, but this handles 'back' or abrupt changes
                            transition={{ duration: 0.2 }}

                            // Drag props
                            drag="x"
                            dragConstraints={{ left: 0, right: 0 }}

                            style={{ x, rotate, touchAction: "none" }}
                            onDragEnd={handleDragEnd}
                            className="w-full h-full max-h-[600px] absolute cursor-grab active:cursor-grabbing"

                            // Long press simulation using Framer Motion gestures
                            onTapStart={() => {
                                longPressTimer.current = setTimeout(handleLongPress, 800);
                            }}
                            onTapCancel={() => {
                                if (longPressTimer.current) clearTimeout(longPressTimer.current);
                            }}
                            onTap={() => {
                                if (longPressTimer.current) clearTimeout(longPressTimer.current);
                            }}
                            onDragStart={() => {
                                if (longPressTimer.current) clearTimeout(longPressTimer.current);
                            }}
                        >
                             {/* Visual Feedback Overlays */}
                             <motion.div
                                className="absolute inset-0 flex items-center justify-center z-50 pointer-events-none"
                                style={{ opacity: opacityRight }}
                             >
                                <div className="text-success p-6 rounded-full border-4 border-success bg-background/80 backdrop-blur-sm">
                                    <CheckIcon />
                                </div>
                             </motion.div>

                             <motion.div
                                className="absolute inset-0 flex items-center justify-center z-50 pointer-events-none"
                                style={{ opacity: opacityLeft }}
                             >
                                <div className="text-danger p-6 rounded-full border-4 border-danger bg-background/80 backdrop-blur-sm">
                                    <CrossIcon />
                                </div>
                             </motion.div>

                            <Card className="w-full h-full shadow-xl bg-content1 border border-default-200">
                                <CardHeader className="flex justify-between items-start pb-0">
                                    <span className="text-default-500 text-sm">
                                        Count: {currentWord.anki_count}
                                    </span>
                                    <Tabs
                                        size="sm"
                                        variant="solid"
                                        color={getPriorityColor(currentWord.priority)}
                                        selectedKey={currentWord.priority.toString()}
                                        onSelectionChange={(e) => {
                                            const newP = parseInt(e.toString());
                                            changePriority(currentWord.word, newP, (err) => {
                                                if (!err) {
                                                    setWordsMap(prev => {
                                                        const newMap = new Map(prev);
                                                        const w = newMap.get(page);
                                                        if (w) w.priority = newP;
                                                        return newMap;
                                                    });
                                                }
                                            });
                                        }}
                                    >
                                        <Tab title="Low" key={0} />
                                        <Tab title="Medium" key={1} />
                                        <Tab title="High" key={2} />
                                    </Tabs>
                                </CardHeader>

                                <CardBody className="flex flex-col items-center pt-8 px-6 text-center overflow-y-auto scrollbar-hide">
                                    <h1 className="text-4xl font-bold mb-6 break-words w-full select-text">
                                        {currentWord.word}
                                    </h1>

                                    <Divider className="my-4" />

                                    <div className="w-full text-left prose max-w-none dark:prose-invert flex-1">
                                        <Spoiler>
                                            {currentWord.example && (
                                                <div className="mb-4 bg-default-50 p-3 rounded-lg">
                                                    <p className="font-semibold text-xs text-default-400 mb-1">EXAMPLE</p>
                                                    <Streamdown rehypePlugins={[rehypeRaw]} remarkPlugins={[remarkGfm]}>
                                                        {currentWord.example}
                                                    </Streamdown>
                                                </div>
                                            )}

                                            <div className="mt-4">
                                                <p className="font-semibold text-xs text-default-400 mb-1">EXPLANATION</p>
                                                <Streamdown rehypePlugins={[rehypeRaw]} remarkPlugins={[remarkGfm]}>
                                                    {currentWord.explain}
                                                </Streamdown>
                                            </div>
                                        </Spoiler>
                                    </div>
                                </CardBody>

                                <div className="p-4 flex justify-between w-full border-t border-default-100">
                                    <Button
                                        color="danger"
                                        variant="flat"
                                        onPress={() => handleSwipe('skip')}
                                    >
                                        Skip
                                    </Button>

                                    <div className="flex gap-2">
                                         <Button
                                            isIconOnly
                                            variant="light"
                                            isDisabled={page <= 1}
                                            onPress={() => setPage(p => Math.max(1, p - 1))}
                                         >
                                            <LeftArrowIcon />
                                         </Button>
                                    </div>

                                    <Button
                                        color="success"
                                        variant="flat"
                                        onPress={() => handleSwipe('know')}
                                    >
                                        Know
                                    </Button>
                                </div>
                            </Card>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </div>
    );
}
