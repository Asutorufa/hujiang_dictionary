import {
  changePriority,
  countWord,
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
  Badge,
  DropdownMenu,
  Spinner,
} from "@radix-ui/themes";
import {
  motion,
  PanInfo,
  useAnimation,
  useDragControls,
  useMotionValue,
  useReducedMotion,
  useTransform,
} from "framer-motion";
import { forwardRef, useCallback, useEffect, useImperativeHandle, useRef, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import { EmptyState } from "@/ui/EmptyState";
import { ArrowLeft, ArrowRight, Check, Layers3, RotateCcw, SlidersHorizontal, X } from "lucide-react";

type ReviewCardMotionProps = {
  currentWord: ListWordResponse;
  page: number;
  total: number;
  shouldReduceMotion: boolean | null;
  onSwipe: (action: "know" | "skip") => Promise<void>;
  onPriorityChange: (key: string) => void;
};

type ReviewCardMotionHandle = {
  swipe: (action: "know" | "skip") => Promise<void>;
};

const ReviewCardMotion = forwardRef<ReviewCardMotionHandle, ReviewCardMotionProps>(function ReviewCardMotion({
  currentWord,
  page,
  total,
  shouldReduceMotion,
  onSwipe,
  onPriorityChange,
}, ref) {
  const controls = useAnimation();
  const dragControls = useDragControls();
  const dragX = useMotionValue(0);
  const opacityRight = useTransform(dragX, [60, 150], [0, 0.85]);
  const opacityLeft = useTransform(dragX, [-150, -60], [0.85, 0]);
  const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isAnimating = useRef(false);

  useEffect(() => {
    controls.stop();
    dragX.set(0);
    controls.set({ x: 0, y: 0, opacity: 1, scale: 1 });
  }, [controls, currentWord, dragX, page]);

  const handleDragEnd = async (
    _event: MouseEvent | TouchEvent | PointerEvent,
    info: PanInfo,
  ) => {
    const threshold = 100;
    dragX.set(0);

    if (info.offset.x > threshold) {
      await handleSwipe("know");
    } else if (info.offset.x < -threshold) {
      await handleSwipe("skip");
    } else {
      await controls.start({ x: 0, y: 0, opacity: 1, scale: 1 });
    }
  };

  const handleSwipe = useCallback(async (action: "know" | "skip") => {
    if (isAnimating.current) return;
    isAnimating.current = true;

    try {
      const canAdvance = page < total;
      const exitX = action === "know" ? 500 : -500;

      if (canAdvance) {
        await controls.start({ x: exitX, opacity: 0 });
      } else {
        await controls.start({ x: exitX > 0 ? 56 : -56, opacity: 0.85 });
      }

      await onSwipe(action);

      if (!canAdvance) {
        await controls.start({ x: 0, y: 0, opacity: 1, scale: 1 });
      }
    } finally {
      isAnimating.current = false;
    }
  }, [controls, onSwipe, page, total]);

  useImperativeHandle(ref, () => ({ swipe: handleSwipe }), [handleSwipe]);

  const handleLongPress = () => {
    navigator.clipboard.writeText(currentWord.word).then(() => {
      addToast({ title: "Copied!", color: "success" });
    });
  };

  return (
    <motion.div
      initial={shouldReduceMotion ? false : { opacity: 1, x: 0, y: 0, scale: 1 }}
      animate={controls}
      transition={{ duration: 0.16 }}
      drag="x"
      dragListener={false}
      dragControls={dragControls}
      dragConstraints={{ left: 0, right: 0 }}
      dragDirectionLock
      dragElastic={shouldReduceMotion ? 0.15 : 0.45}
      onDrag={(_, info) => dragX.set(info.offset.x)}
      onDragEnd={handleDragEnd}
      style={{ touchAction: "pan-y" }}
      className="app-review-card-wrap"
      onTapStart={() => { longPressTimer.current = setTimeout(handleLongPress, 800); }}
      onTapCancel={() => { if (longPressTimer.current) clearTimeout(longPressTimer.current); }}
      onTap={() => { if (longPressTimer.current) clearTimeout(longPressTimer.current); }}
      onDragStart={() => { if (longPressTimer.current) clearTimeout(longPressTimer.current); }}
    >
      <motion.div className="app-review-swipe-feedback app-review-swipe-feedback-right" style={{ opacity: opacityRight }}><Check size={42} /></motion.div>
      <motion.div className="app-review-swipe-feedback app-review-swipe-feedback-left" style={{ opacity: opacityLeft }}><X size={42} /></motion.div>

      <article
        className="app-review-card"
        onPointerDown={(event) => {
          const target = event.target as HTMLElement;
          if (target.closest("a,button,input,textarea,select,[role='menuitem'],[data-radix-collection-item]")) return;
          dragControls.start(event);
        }}
      >
        <header className="app-review-card-header">
          <div>
            <h2>{currentWord.word}</h2>
            <span>Added {new Date(currentWord.update_time * 1000).toLocaleDateString()}</span>
          </div>
          <div className="app-review-card-meta">
            <span>Review {new Date(currentWord.reminder_time * 1000).toLocaleDateString()}</span>
            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger>
                <Badge size="1" variant="soft" color={currentWord.priority === 0 ? "green" : currentWord.priority === 1 ? "orange" : "red"}>
                  {getPriorityText(currentWord.priority)}
                </Badge>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Item color="green" onSelect={() => onPriorityChange("0")}>Low</DropdownMenu.Item>
                <DropdownMenu.Item color="orange" onSelect={() => onPriorityChange("1")}>Medium</DropdownMenu.Item>
                <DropdownMenu.Item color="red" onSelect={() => onPriorityChange("2")}>High</DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </div>
        </header>

        {currentWord.example && (
          <section className="app-review-example prose"><Markdown>{currentWord.example}</Markdown></section>
        )}
        <section className="app-review-meaning">
          <div className="app-word-section-label">Meaning</div>
          <Spoiler flex className="app-review-spoiler" containerClassName="!p-0">
            <div className="prose app-review-meaning-content"><Markdown>{currentWord.explain}</Markdown></div>
          </Spoiler>
        </section>

        <footer className="app-review-card-footer">
          <span><RotateCcw size={14} /> Long press to copy</span>
          <span>Card {page} / {total}</span>
        </footer>
      </article>
    </motion.div>
  );
});

export default function Flashcard() {
  const [page, setPage] = useLocalStorage<number>("flashcard_page_v2", 1);
  const [total, setTotal] = useLocalStorage<number>("flashcard_total_words", 0);
  const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");
  const [grammar, setGrammar] = useLocalStorage("grammar", false);

  const [wordsMap, setWordsMap] = useState<Map<number, ListWordResponse>>(
    new Map(),
  );
  const [loading, setLoading] = useState(false);
  const loadedChunksRef = useRef<Set<number>>(new Set());
  const shouldReduceMotion = useReducedMotion();

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
      if (size !== undefined) {
        setTotal(size);
        if (size === 0 && page !== 1) {
          setPage(1);
        } else if (size > 0 && page > size) {
          setPage(size);
        }
      }
    });
  }, [page, grammar, orderBy, fetchChunksIfNeeded, setPage, setTotal]);

  const currentWord = wordsMap.get(page);
  const reviewCardRef = useRef<ReviewCardMotionHandle>(null);

  const handleSwipe = useCallback(
    async (action: "know" | "skip") => {
      if (action === "know") {
        if (currentWord) {
          await incrementRemindCount(currentWord.word, () => {});
        }
      }

      if (page < total) {
        setPage((p) => Math.min(p + 1, total));
      }
    },
    [currentWord, page, setPage, total],
  );

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
    <div className="app-review-page">
      <header className="app-review-header">
        <div>
          <div className="app-library-eyebrow">Work / Practice</div>
          <h1>Daily review</h1>
          <p>Move through your saved words at your own pace.</p>
        </div>
        <div className="app-review-progress">
          <div className="app-review-progress-copy">
            <strong>{total > 0 ? page : 0}</strong>
            <span>of {total || 0} cards</span>
          </div>
          <div className="app-review-progress-track" aria-hidden="true">
            <span style={{ width: `${total > 0 ? Math.min((page / total) * 100, 100) : 0}%` }} />
          </div>
        </div>
      </header>

      <div className="app-review-toolbar">
        <DropdownMenu.Root modal={false}>
          <DropdownMenu.Trigger asChild disabled={loading && wordsMap.size === 0}>
            <Button variant="ghost" className="app-review-control">
              <SlidersHorizontal size={16} /> {orderBy.replace("_", " ")}
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.Label>Sort review by</DropdownMenu.Label>
            {[
              ["word", "Word"],
              ["word desc", "Word descending"],
              ["priority desc", "Priority"],
              ["reminder_time", "Reminder date"],
              ["anki_count desc", "Review count"],
            ].map(([value, label]) => (
              <DropdownMenu.RadioItem key={value} value={value} onSelect={() => { setOrderBy(value); setPage(1); }}>
                {label}
              </DropdownMenu.RadioItem>
            ))}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
        <DropdownMenu.Root modal={false}>
          <DropdownMenu.Trigger asChild disabled={loading && wordsMap.size === 0}>
            <Button variant="ghost" className="app-review-control">
              <Layers3 size={16} /> {grammar ? "Grammar" : "Words"}
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.RadioItem value="word" onSelect={() => { setGrammar(false); setPage(1); }}>Words</DropdownMenu.RadioItem>
            <DropdownMenu.RadioItem value="grammar" onSelect={() => { setGrammar(true); setPage(1); }}>Grammar</DropdownMenu.RadioItem>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
        <span className="app-review-toolbar-hint">Drag a card or use the buttons below.</span>
      </div>

      <div className="app-review-stage">
        {loading && wordsMap.size === 0 && <Spinner size="3" />}
        {!loading && wordsMap.size === 0 && (
          <EmptyState
            title="No words found"
            description="Add some words first, then come back to review."
            icon={<Layers3 size={28} />}
            actionLabel="Reset"
            onAction={() => setPage(1)}
          />
        )}

        {currentWord && (
          <ReviewCardMotion
            key={page}
            ref={reviewCardRef}
            currentWord={currentWord}
            page={page}
            total={total}
            shouldReduceMotion={shouldReduceMotion}
            onSwipe={handleSwipe}
            onPriorityChange={handlePriorityChange}
          />
        )}
      </div>

      {currentWord && (
        <div className="app-review-actions">
          <Button className="app-review-skip" variant="ghost" onClick={() => reviewCardRef.current?.swipe("skip")}>
            <X size={17} /> Skip
          </Button>
          <button type="button" className="app-review-back" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))} aria-label="Previous card">
            <ArrowLeft size={18} />
          </button>
          <Button className="app-review-know" onClick={() => reviewCardRef.current?.swipe("know")}>
            Know <ArrowRight size={17} />
          </Button>
        </div>
      )}
    </div>
  );
  /*
  return (
    <div className="app-page-shell relative flex h-[100dvh] flex-col items-center overflow-x-visible overflow-y-hidden p-4 pb-[calc(env(safe-area-inset-bottom)+112px)]">
      Header / Filter Bar
      <div className="app-bottom-actions mb-4 flex w-full max-w-md flex-wrap items-center justify-between gap-2 px-3 py-3 z-10">
        <DropdownMenu.Root modal={false}>
          <DropdownMenu.Trigger
            disabled={loading && wordsMap.size === 0}
            className="cursor-pointer"
          >
            <Button
              variant="surface"
              color="gray"
              className="app-control-trigger capitalize"
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

        <DropdownMenu.Root modal={false}>
          <DropdownMenu.Trigger
            disabled={loading && wordsMap.size === 0}
            className="cursor-pointer"
          >
            <Button
              variant="surface"
              color="gray"
              className="app-control-trigger capitalize"
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

        <Flex align="center" ml="2" px="3" py="1" className="app-stat-chip">
          <Text size="2">
            {page} / {total}
          </Text>
        </Flex>
      </div>

      Main Content Area
      <div className="relative flex min-h-0 flex-1 w-full max-w-md items-center justify-center overflow-visible">
        {loading && wordsMap.size === 0 && <Spinner size="3" />}

        {!loading && wordsMap.size === 0 && (
          <EmptyState
            title="No words found"
            description="Add some words first, then come back to review."
            icon={<Layers3 size={28} className="text-[var(--gray-a11)]" />}
            actionLabel="Reset"
            onAction={() => setPage(1)}
          />
        )}

        {currentWord && (
          <motion.div
            key={page}
            initial={
              shouldReduceMotion ? false : { opacity: 0, y: 24, scale: 0.98 }
            }
            animate={controls}
            transition={{ duration: 0.16 }}
            drag="x"
            dragListener={false}
            dragControls={dragControls}
            dragConstraints={{ left: 0, right: 0 }}
            dragDirectionLock
            dragElastic={shouldReduceMotion ? 0.15 : 0.45}
            onDragEnd={handleDragEnd}
            style={{ x, rotate, touchAction: "pan-y" }}
            className="w-full absolute inset-0 cursor-grab active:cursor-grabbing flex flex-col min-h-0"
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
            Visual Feedback Overlays
            <motion.div
              className="absolute inset-0 flex items-center justify-center z-50 pointer-events-none"
              style={{ opacity: opacityRight }}
            >
              <div className="text-green-500 p-6 rounded-full border-4 border-green-500 bg-[color-mix(in_srgb,var(--color-panel-solid)_92%,transparent)]">
                <CheckIcon />
              </div>
            </motion.div>

            <motion.div
              className="absolute inset-0 flex items-center justify-center z-50 pointer-events-none"
              style={{ opacity: opacityLeft }}
            >
              <div className="text-red-500 p-6 rounded-full border-4 border-red-500 bg-[color-mix(in_srgb,var(--color-panel-solid)_92%,transparent)]">
                <CrossIcon />
              </div>
            </motion.div>

            <Card
              className="app-section-card w-full h-full overflow-hidden relative"
              style={{ padding: 0 }}
              onPointerDown={(e) => {
                const target = e.target as HTMLElement;
                if (
                  target.closest(
                    "a,button,input,textarea,select,[role='menuitem'],[data-radix-collection-item]",
                  )
                ) {
                  return;
                }
                dragControls.start(e);
              }}
            >
              <div className="absolute inset-0 flex flex-col p-4">
                Header
                <Flex justify="between" align="start" flexShrink="0">
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
                    <DropdownMenu.Root modal={false}>
                      <DropdownMenu.Trigger className="cursor-pointer">
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

                Content Area
                <div
                  className="flex-1 flex flex-col min-h-0 mt-3 overscroll-contain"
                  onPointerDown={(e) => {
                    e.stopPropagation();
                  }}
                >
                  {currentWord.example && (
                    <Box className="app-muted-panel mb-3 p-3 shrink-0 prose prose-sm max-w-none dark:prose-invert">
                      <Markdown>{currentWord.example}</Markdown>
                    </Box>
                  )}

                  <Spoiler
                    flex
                    className="flex-1 min-h-0"
                    containerClassName="!p-0"
                  >
                    <div className="flex-1 flex flex-col min-h-0">
                      <div className="flex-1 overflow-y-auto custom-scrollbar pr-1 prose prose-sm max-w-none dark:prose-invert">
                        <Markdown>{currentWord.explain}</Markdown>
                      </div>
                    </div>
                  </Spoiler>
                </div>
              </div>
            </Card>
          </motion.div>
        )}
      </div>

      Fixed Bottom Buttons — always visible
      {currentWord && (
        <div className="app-bottom-actions w-full max-w-md py-3 px-3 flex items-center justify-between gap-3 z-10">
          <Button
            color="gray"
            variant="soft"
            className="app-secondary-action flex-1"
            onClick={() => handleSwipe("skip")}
          >
            Skip
          </Button>

          <IconButton
            variant="ghost"
            color="gray"
            disabled={page <= 1}
            className="app-icon-chip"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
          >
            <LeftArrowIcon />
          </IconButton>

          <Button
            color="gray"
            variant="solid"
            className="app-primary-action flex-1"
            onClick={() => handleSwipe("know")}
          >
            Know
          </Button>
        </div>
      )}
    </div>
  );
  */
}
