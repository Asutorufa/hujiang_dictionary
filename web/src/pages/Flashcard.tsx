import {
  BookIcon,
  changePriority,
  countWord,
  FilterIcon,
  getPriorityText,
  incrementRemindCount,
  ListWordResponse,
  Markdown,
  queryWord,
  Spoiler,
} from "@/components";
import { addToast } from "@/components";
import {
  Button,
  Card,
  Badge,
  DropdownMenu,
  Spinner,
  IconButton,
  Flex,
  Text,
  Box,
} from "@radix-ui/themes";
import {
  AnimatePresence,
  motion,
  PanInfo,
  useAnimation,
  useDragControls,
  useMotionValue,
  useTransform,
} from "framer-motion";
import { useCallback, useEffect, useRef, useState } from "react";
import { useLocalStorage } from "usehooks-ts";

// Helper for swipe icons
const CheckIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="100"
    height="100"
    viewBox="0 0 24 24"
  >
    <path
      fill="currentColor"
      d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10s10-4.48 10-10S17.52 2 12 2m-2 15l-5-5l1.41-1.41L10 14.17l7.59-7.59L19 8z"
    />
  </svg>
);

const CrossIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="100"
    height="100"
    viewBox="0 0 24 24"
  >
    <path
      fill="currentColor"
      d="M12 2C6.47 2 2 6.47 2 12s4.47 10 10 10s10-4.47 10-10S17.53 2 12 2m5 13.59L15.59 17L12 13.41L8.41 17L7 15.59L10.59 12L7 8.41L8.41 7L12 10.59L15.59 7L17 8.41L13.41 12z"
    />
  </svg>
);

const LeftArrowIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
  >
    <path
      fill="currentColor"
      d="M15.41 7.41L14 6l-6 6l6 6l1.41-1.41L10.83 12z"
    />
  </svg>
);

export default function Flashcard() {
  const [page, setPage] = useLocalStorage<number>("flashcard_page_v2", 1);
  const [total, setTotal] = useLocalStorage<number>("total_page", 100);
  const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");
  const [grammar, setGrammar] = useLocalStorage("grammar", false);

  const [wordsMap, setWordsMap] = useState<Map<number, ListWordResponse>>(
    new Map(),
  );
  const [loading, setLoading] = useState(false);
  const loadedChunksRef = useRef<Set<number>>(new Set());
  const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const CHUNK_SIZE = 10;

  const fetchChunksIfNeeded = useCallback(
    async (targetIndex: number) => {
      const targetPage = Math.ceil(targetIndex / CHUNK_SIZE);

      const neededChunks: number[] = [];

      if (!loadedChunksRef.current.has(targetPage)) {
        neededChunks.push(targetPage);
      }

      if (targetIndex % CHUNK_SIZE > 5) {
        if (!loadedChunksRef.current.has(targetPage + 1)) {
          neededChunks.push(targetPage + 1);
        }
      }

      if (neededChunks.length > 0) {
        setLoading(true);
        neededChunks.forEach((c) => loadedChunksRef.current.add(c));

        await Promise.all(
          neededChunks.map(
            (p) =>
              new Promise<void>((resolve) => {
                queryWord(p, CHUNK_SIZE, orderBy, grammar, (data) => {
                  if (data) {
                    setWordsMap((prev) => {
                      const newMap = new Map(prev);
                      data.forEach((w, i) => {
                        const absIndex = (p - 1) * CHUNK_SIZE + 1 + i;
                        newMap.set(absIndex, w);
                      });
                      return newMap;
                    });
                  }
                  resolve();
                });
              }),
          ),
        );
        setLoading(false);
      }
    },
    [orderBy, grammar, CHUNK_SIZE],
  );

  useEffect(() => {
    setWordsMap(new Map());
    loadedChunksRef.current.clear();
    fetchChunksIfNeeded(page);
  }, [orderBy, grammar, fetchChunksIfNeeded, page]);

  useEffect(() => {
    fetchChunksIfNeeded(page);
    countWord(grammar, (size) => {
      if (size) setTotal(size);
    });
  }, [page, grammar, orderBy, fetchChunksIfNeeded, setTotal]);

  const currentWord = wordsMap.get(page);

  const controls = useAnimation();
  const dragControls = useDragControls();
  const x = useMotionValue(0);
  const rotate = useTransform(x, [-200, 200], [-10, 10]);
  const opacityRight = useTransform(x, [50, 150], [0, 1]);
  const opacityLeft = useTransform(x, [-150, -50], [1, 0]);

  const handleSwipe = useCallback(
    async (action: "know" | "skip") => {
      if (action === "know") {
        await controls.start({ x: 500, opacity: 0 });
        if (currentWord) {
          await incrementRemindCount(currentWord.word, () => {});
        }
      } else {
        await controls.start({ x: -500, opacity: 0 });
      }
      setPage((p) => Math.min(p + 1, total));
    },
    [controls, currentWord, setPage, total],
  );

  const handleDragEnd = async (
    _event: MouseEvent | TouchEvent | PointerEvent,
    info: PanInfo,
  ) => {
    const threshold = 100;
    if (info.offset.x > threshold) {
      await handleSwipe("know");
    } else if (info.offset.x < -threshold) {
      await handleSwipe("skip");
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

  const handlePriorityChange = (key: string) => {
    if (!currentWord) return;
    const newP = parseInt(key);
    changePriority(currentWord.word, newP, (err) => {
      if (!err) {
        setWordsMap((prev) => {
          const newMap = new Map(prev);
          const w = newMap.get(page);
          if (w) w.priority = newP;
          return newMap;
        });
      }
    });
  };

  return (
    <div className="flex flex-col h-[calc(100vh-80px)] overflow-hidden items-center relative p-4">
      {/* Header / Filter Bar */}
      <div className="flex gap-2 mb-4 z-10 w-full justify-center">
        <DropdownMenu.Root>
          <DropdownMenu.Trigger disabled={loading && wordsMap.size === 0}>
            <Button
              variant="surface"
              color="gray"
              className="shadow-md backdrop-blur-sm capitalize"
            >
              <FilterIcon /> {orderBy.replace("_", " ")}
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.RadioGroup
              value={orderBy}
              onValueChange={(val) => {
                setOrderBy(val);
                setPage(1);
              }}
            >
              <DropdownMenu.RadioItem value="word">Word</DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="word desc">
                Word DESC
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="priority">
                Priority
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="priority desc">
                Priority DESC
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="add_time">
                Add Time
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="add_time desc">
                Add Time DESC
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="update_time">
                Update Time
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="update_time desc">
                Update Time DESC
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="reminder_time">
                Reminder
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="reminder_time desc">
                Reminder DESC
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="anki_count">
                Count
              </DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="anki_count desc">
                Count DESC
              </DropdownMenu.RadioItem>
            </DropdownMenu.RadioGroup>
          </DropdownMenu.Content>
        </DropdownMenu.Root>

        <DropdownMenu.Root>
          <DropdownMenu.Trigger disabled={loading && wordsMap.size === 0}>
            <Button
              variant="surface"
              color="gray"
              className="shadow-md backdrop-blur-sm capitalize"
            >
              <BookIcon /> {grammar ? "Grammar" : "Word"}
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.RadioGroup
              value={grammar ? "grammar" : "word"}
              onValueChange={(val) => {
                setGrammar(val === "grammar");
                setPage(1);
              }}
            >
              <DropdownMenu.RadioItem value="word">Word</DropdownMenu.RadioItem>
              <DropdownMenu.RadioItem value="grammar">
                Grammar
              </DropdownMenu.RadioItem>
            </DropdownMenu.RadioGroup>
          </DropdownMenu.Content>
        </DropdownMenu.Root>

        <Flex
          align="center"
          ml="2"
          px="3"
          py="1"
          className="bg-default-100 rounded-lg"
        >
          <Text size="2">
            {page} / {total}
          </Text>
        </Flex>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 w-full max-w-md flex items-center justify-center relative overflow-hidden">
        {loading && wordsMap.size === 0 && <Spinner size="3" />}

        {!loading && wordsMap.size === 0 && (
          <div className="text-center">
            <Text color="gray">No words found.</Text>
            <Button className="mt-4" onClick={() => setPage(1)}>
              Reset to start
            </Button>
          </div>
        )}

        <AnimatePresence mode="wait">
          {currentWord && (
            <motion.div
              key={page}
              initial={{ opacity: 0, y: 50, scale: 0.9 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, x: 0 }}
              transition={{ duration: 0.2 }}
              drag="x"
              dragControls={dragControls}
              dragConstraints={{ left: 0, right: 0 }}
              dragDirectionLock
              dragElastic={0.9}
              onDragEnd={handleDragEnd}
              style={{ x, rotate, touchAction: "pan-y" }}
              className="w-full absolute inset-0 cursor-grab active:cursor-grabbing flex flex-col"
              onTapStart={() => {
                longPressTimer.current = setTimeout(handleLongPress, 800);
              }}
              onTapCancel={() => {
                if (longPressTimer.current)
                  clearTimeout(longPressTimer.current);
              }}
              onTap={() => {
                if (longPressTimer.current)
                  clearTimeout(longPressTimer.current);
              }}
              onDragStart={() => {
                if (longPressTimer.current)
                  clearTimeout(longPressTimer.current);
              }}
            >
              {/* Visual Feedback Overlays */}
              <motion.div
                className="absolute inset-0 flex items-center justify-center z-50 pointer-events-none"
                style={{ opacity: opacityRight }}
              >
                <div className="text-green-500 p-6 rounded-full border-4 border-green-500 bg-[var(--color-panel-solid)]/80 backdrop-blur-sm">
                  <CheckIcon />
                </div>
              </motion.div>

              <motion.div
                className="absolute inset-0 flex items-center justify-center z-50 pointer-events-none"
                style={{ opacity: opacityLeft }}
              >
                <div className="text-red-500 p-6 rounded-full border-4 border-red-500 bg-[var(--color-panel-solid)]/80 backdrop-blur-sm">
                  <CrossIcon />
                </div>
              </motion.div>

              <Card className="flex-1 flex flex-col overflow-hidden">
                <Flex justify="between" align="start">
                  <Flex direction="column">
                    <Text size="5" weight="bold" className="break-words">
                      {currentWord.word}
                    </Text>
                    <Text size="1" color="gray">
                      {new Date(
                        currentWord.update_time * 1000,
                      ).toLocaleDateString()}
                    </Text>
                  </Flex>

                  <Flex align="center" gap="2">
                    <Text size="1" color="gray">
                      Review:{" "}
                      {new Date(
                        currentWord.reminder_time * 1000,
                      ).toLocaleDateString()}
                    </Text>
                    <DropdownMenu.Root>
                      <DropdownMenu.Trigger>
                        <Badge
                          size="1"
                          variant="soft"
                          color={
                            currentWord.priority === 0
                              ? "green"
                              : currentWord.priority === 1
                                ? "orange"
                                : "gray"
                          }
                          className="cursor-pointer"
                        >
                          {getPriorityText(currentWord.priority)}
                        </Badge>
                      </DropdownMenu.Trigger>
                      <DropdownMenu.Content>
                        <DropdownMenu.Item
                          color="green"
                          onSelect={() => handlePriorityChange("0")}
                        >
                          Low
                        </DropdownMenu.Item>
                        <DropdownMenu.Item
                          color="orange"
                          onSelect={() => handlePriorityChange("1")}
                        >
                          Medium
                        </DropdownMenu.Item>
                        <DropdownMenu.Item
                          color="gray"
                          onSelect={() => handlePriorityChange("2")}
                        >
                          High
                        </DropdownMenu.Item>
                      </DropdownMenu.Content>
                    </DropdownMenu.Root>
                  </Flex>
                </Flex>

                <Box
                  className="flex-1 overflow-y-auto overflow-x-hidden mt-3"
                  onPointerDown={(e: React.PointerEvent<HTMLDivElement>) => {
                    const target = e.target as HTMLElement;
                    if (target.closest(".prose")) return;
                    dragControls.start(e);
                  }}
                >
                  <div className="w-full text-left prose prose-sm max-w-none dark:prose-invert">
                    {currentWord.example && (
                      <Box className="rounded-lg bg-[var(--gray-a3)] p-3 mb-2">
                        <Markdown>{currentWord.example}</Markdown>
                      </Box>
                    )}

                    <Spoiler>
                      <Box mt="2">
                        <Markdown>{currentWord.explain}</Markdown>
                      </Box>
                    </Spoiler>
                  </div>
                </Box>
              </Card>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Fixed Bottom Buttons — always visible */}
      {currentWord && (
        <div className="w-full max-w-md py-3 flex items-center justify-between gap-3 z-10">
          <Button
            color="red"
            variant="soft"
            className="flex-1 cursor-pointer"
            onClick={() => handleSwipe("skip")}
          >
            Skip
          </Button>

          <IconButton
            variant="ghost"
            color="gray"
            disabled={page <= 1}
            className="cursor-pointer"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
          >
            <LeftArrowIcon />
          </IconButton>

          <Button
            color="green"
            variant="soft"
            className="flex-1 cursor-pointer"
            onClick={() => handleSwipe("know")}
          >
            Know
          </Button>
        </div>
      )}
    </div>
  );
}
