"use client";

import { BookIcon, changePriority, countWord, FilterIcon, getPriorityColor, incrementRemindCount, ListWordResponse, queryWord } from "@/app/components";
import { Button, Card, CardBody, CardHeader, Divider, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, Spinner, Tab, Tabs } from "@heroui/react";
import { motion, PanInfo, useAnimation, useMotionValue, useTransform } from "framer-motion";
import { useEffect, useState } from "react";
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

export default function Flashcard() {
    const [wordData, setWordData] = useState<ListWordResponse | null>(null);
    const [page, setPage] = useLocalStorage<number>("flashcard_page", 1);
    const [total, setTotal] = useLocalStorage<number>("total_page", 100);

    const [loading, setLoading] = useState(false);
    const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");
    const [grammar, setGrammar] = useLocalStorage("grammar", false);

    const controls = useAnimation();
    const x = useMotionValue(0);
    const rotate = useTransform(x, [-200, 200], [-10, 10]);
    const opacityRight = useTransform(x, [50, 150], [0, 1]);
    const opacityLeft = useTransform(x, [-150, -50], [1, 0]);
    const scale = useTransform(x, [-200, 0, 200], [0.9, 1, 0.9]);

    // Fetch total count to know limits
    useEffect(() => {
        countWord(grammar, (size) => {
            if (size) {
                setTotal(size);
                if (page > size && size > 0) setPage(1);
            }
        });
    }, [grammar, page, setPage, setTotal]);

    // Fetch current word
    useEffect(() => {
        setLoading(true);
        setWordData(null);
        x.set(0);
        controls.set({ x: 0, opacity: 1, scale: 1, rotate: 0 });

        queryWord(page, 1, orderBy, grammar, (data) => {
            if (data && data.length > 0) {
                setWordData(data[0]);
            } else {
                setWordData(null);
            }
            setLoading(false);
        });
    }, [page, orderBy, grammar, controls, x]);

    const handleDragEnd = async (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => {
        const threshold = 100;
        if (info.offset.x > threshold) {
            // Swipe Right - Remember
            await controls.start({ x: 500, opacity: 0 });
            if (wordData) {
                await incrementRemindCount(wordData.word, () => {});
            }
            setPage(p => p + 1);
        } else if (info.offset.x < -threshold) {
            // Swipe Left - Skip/Forgot
            await controls.start({ x: -500, opacity: 0 });
            setPage(p => p + 1);
        } else {
            // Reset
            controls.start({ x: 0, rotate: 0, scale: 1 });
        }
    };

    return (
        <div className="flex flex-col h-[calc(100vh-80px)] overflow-hidden items-center relative p-4">
            {/* Header / Filter Bar */}
            <div className="flex gap-2 mb-4 z-10 w-full justify-center">
                <Dropdown>
                    <DropdownTrigger>
                        <Button isDisabled={loading} variant="bordered" className="shadow-md backdrop-blur-sm capitalize">
                            <FilterIcon /> {orderBy.replace("_", " ")}
                        </Button>
                    </DropdownTrigger>
                    <DropdownMenu
                        selectionMode="single"
                        selectedKeys={[orderBy]}
                        onSelectionChange={(e) => {
                            const val = e.currentKey ? String(e.currentKey) : "word";
                            setOrderBy(val);
                            setPage(1); // Reset to start on sort change
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
                        <Button isDisabled={loading} variant="bordered" className="shadow-md backdrop-blur-sm capitalize">
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
                {loading && <Spinner size="lg" />}

                {!loading && !wordData && (
                    <div className="text-center text-default-500">
                        <p>No words found.</p>
                        <Button className="mt-4" onPress={() => setPage(1)}>Reset to start</Button>
                    </div>
                )}

                {!loading && wordData && (
                    <motion.div
                        drag="x"
                        dragConstraints={{ left: 0, right: 0 }}
                        animate={controls}
                        style={{ x, rotate, scale, touchAction: "none" }}
                        onDragEnd={handleDragEnd}
                        className="w-full h-full max-h-[600px] absolute cursor-grab active:cursor-grabbing"
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
                                    Count: {wordData.anki_count}
                                </span>
                                <Tabs
                                    size="sm"
                                    variant="solid"
                                    color={getPriorityColor(wordData.priority)}
                                    selectedKey={wordData.priority.toString()}
                                    onSelectionChange={(e) => {
                                        const newP = parseInt(e.toString());
                                        changePriority(wordData.word, newP, (err) => {
                                            if (!err) {
                                                setWordData({ ...wordData, priority: newP });
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
                                <h1 className="text-4xl font-bold mb-6 break-words w-full">
                                    {wordData.word}
                                </h1>

                                <Divider className="my-4" />

                                <div className="w-full text-left prose max-w-none dark:prose-invert flex-1">
                                    {wordData.example && (
                                        <div className="mb-4 bg-default-50 p-3 rounded-lg">
                                            <p className="font-semibold text-xs text-default-400 mb-1">EXAMPLE</p>
                                            <Streamdown rehypePlugins={[rehypeRaw]} remarkPlugins={[remarkGfm]}>
                                                {wordData.example}
                                            </Streamdown>
                                        </div>
                                    )}

                                    <div className="mt-4">
                                        <p className="font-semibold text-xs text-default-400 mb-1">EXPLANATION</p>
                                        <Streamdown rehypePlugins={[rehypeRaw]} remarkPlugins={[remarkGfm]}>
                                            {wordData.explain}
                                        </Streamdown>
                                    </div>
                                </div>
                            </CardBody>

                            {/* Mobile/Desktop helper instructions or buttons if drag is hard */}
                            <div className="p-4 flex justify-between w-full border-t border-default-100">
                                <Button
                                    color="danger"
                                    variant="flat"
                                    onPress={() => {
                                        controls.start({ x: -500, opacity: 0 }).then(() => setPage(p => p + 1));
                                    }}
                                >
                                    Skip
                                </Button>
                                <span className="text-xs text-default-400 flex items-center">
                                    Swipe Left/Right
                                </span>
                                <Button
                                    color="success"
                                    variant="flat"
                                    onPress={() => {
                                        controls.start({ x: 500, opacity: 0 }).then(() => {
                                            incrementRemindCount(wordData.word, () => {});
                                            setPage(p => p + 1);
                                        });
                                    }}
                                >
                                    Know
                                </Button>
                            </div>
                        </Card>
                    </motion.div>
                )}
            </div>
        </div>
    );
}
