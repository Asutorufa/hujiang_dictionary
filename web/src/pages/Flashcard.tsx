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
import { Button, Badge, DropdownMenu, Spinner } from "@radix-ui/themes";
import {
  motion,
  PanInfo,
  useAnimation,
  useDragControls,
  useMotionValue,
  useReducedMotion,
  useTransform,
} from "motion/react";
import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import { useLocalStorage } from "usehooks-ts";
import { EmptyState } from "@/ui/EmptyState";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  Layers3,
  RotateCcw,
  SlidersHorizontal,
  X,
} from "lucide-react";

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

const ReviewCardMotion = forwardRef<
  ReviewCardMotionHandle,
  ReviewCardMotionProps
>(function ReviewCardMotion(
  { currentWord, page, total, shouldReduceMotion, onSwipe, onPriorityChange },
  ref,
) {
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
    if (shouldReduceMotion) {
      controls.set({ x: 0, y: 0, opacity: 1, scale: 1, rotate: 0 });
      return;
    }
    // Bring the next card in softly instead of popping it into place after
    // the previous one has been swiped away.
    controls.set({ x: 0, y: 10, opacity: 0, scale: 0.985, rotate: 0 });
    void controls.start({
      y: 0,
      opacity: 1,
      scale: 1,
      transition: { duration: 0.2, ease: [0.22, 0.61, 0.36, 1] },
    });
  }, [controls, currentWord, dragX, page, shouldReduceMotion]);

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

  const handleSwipe = useCallback(
    async (action: "know" | "skip") => {
      if (isAnimating.current) return;
      isAnimating.current = true;

      try {
        const canAdvance = page < total;
        const exitX = action === "know" ? 500 : -500;

        if (canAdvance) {
          await controls.start({
            x: exitX,
            opacity: 0,
            rotate: shouldReduceMotion ? 0 : exitX > 0 ? 4 : -4,
            transition: { duration: 0.2, ease: [0.4, 0, 1, 1] },
          });
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
    },
    [controls, onSwipe, page, total, shouldReduceMotion],
  );

  useImperativeHandle(ref, () => ({ swipe: handleSwipe }), [handleSwipe]);

  const handleLongPress = () => {
    navigator.clipboard.writeText(currentWord.word).then(() => {
      addToast({ title: "Copied!", color: "success" });
    });
  };

  return (
    <motion.div
      initial={
        shouldReduceMotion ? false : { opacity: 1, x: 0, y: 0, scale: 1 }
      }
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
      <motion.div
        className="app-review-swipe-feedback app-review-swipe-feedback-right"
        style={{ opacity: opacityRight }}
      >
        <Check size={42} />
      </motion.div>
      <motion.div
        className="app-review-swipe-feedback app-review-swipe-feedback-left"
        style={{ opacity: opacityLeft }}
      >
        <X size={42} />
      </motion.div>

      <article
        className="app-review-card"
        onPointerDown={(event) => {
          const target = event.target as HTMLElement;
          if (
            target.closest(
              "a,button,input,textarea,select,[role='menuitem'],[data-radix-collection-item]",
            )
          )
            return;
          dragControls.start(event);
        }}
      >
        <header className="app-review-card-header">
          <div>
            <h2>{currentWord.word}</h2>
            <span>
              Added{" "}
              {new Date(currentWord.update_time * 1000).toLocaleDateString()}
            </span>
          </div>
          <div className="app-review-card-meta">
            <span>
              Review{" "}
              {new Date(currentWord.reminder_time * 1000).toLocaleDateString()}
            </span>
            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger>
                <Badge
                  size="1"
                  variant="soft"
                  color={
                    currentWord.priority === 0
                      ? "green"
                      : currentWord.priority === 1
                        ? "orange"
                        : "red"
                  }
                >
                  {getPriorityText(currentWord.priority)}
                </Badge>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Item
                  color="green"
                  onSelect={() => onPriorityChange("0")}
                >
                  Low
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  color="orange"
                  onSelect={() => onPriorityChange("1")}
                >
                  Medium
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  color="red"
                  onSelect={() => onPriorityChange("2")}
                >
                  High
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </div>
        </header>

        {currentWord.example && (
          <section className="app-review-example prose">
            <Markdown>{currentWord.example}</Markdown>
          </section>
        )}
        <section className="app-review-meaning">
          <div className="app-word-section-label">Meaning</div>
          <Spoiler
            flex
            className="app-review-spoiler"
            containerClassName="!p-0"
          >
            <div className="prose app-review-meaning-content">
              <Markdown>{currentWord.explain}</Markdown>
            </div>
          </Spoiler>
        </section>

        <footer className="app-review-card-footer">
          <span>
            <RotateCcw size={14} /> Long press to copy
          </span>
          <span>
            Card {page} / {total}
          </span>
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

  // Changing the sort order or the word type invalidates the cached chunks.
  // (Previously this also ran on every page change, which wiped the cache and
  // refetched on each swipe, making the card flash between words.)
  useEffect(() => {
    setWordsMap(new Map());
    loadedChunksRef.current.clear();
  }, [fetchChunksIfNeeded]);

  useEffect(() => {
    fetchChunksIfNeeded(page);
  }, [page, fetchChunksIfNeeded]);

  useEffect(() => {
    countWord(grammar, (size) => {
      if (size === undefined) return;
      setTotal(size);
      setPage((current) => (size === 0 ? 1 : current > size ? size : current));
    });
  }, [grammar, orderBy, setPage, setTotal]);

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
          <h1>Daily review</h1>
          <p>Move through your saved words at your own pace.</p>
        </div>
        <div className="app-review-progress">
          <div className="app-review-progress-copy">
            <strong>{total > 0 ? page : 0}</strong>
            <span>of {total || 0} cards</span>
          </div>
          <div className="app-review-progress-track" aria-hidden="true">
            <span
              style={{
                transform: `scaleX(${total > 0 ? Math.min(page / total, 1) : 0})`,
              }}
            />
          </div>
        </div>
      </header>

      <div className="app-review-toolbar">
        <DropdownMenu.Root modal={false}>
          <DropdownMenu.Trigger disabled={loading && wordsMap.size === 0}>
            <button type="button" className="app-review-control">
              <SlidersHorizontal size={16} />
              <span>{orderBy.replace("_", " ")}</span>
            </button>
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
              <DropdownMenu.RadioItem
                key={value}
                value={value}
                onSelect={() => {
                  setOrderBy(value);
                  setPage(1);
                }}
              >
                {label}
              </DropdownMenu.RadioItem>
            ))}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
        <DropdownMenu.Root modal={false}>
          <DropdownMenu.Trigger disabled={loading && wordsMap.size === 0}>
            <button type="button" className="app-review-control">
              <Layers3 size={16} />
              <span>{grammar ? "Grammar" : "Words"}</span>
            </button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.RadioItem
              value="word"
              onSelect={() => {
                setGrammar(false);
                setPage(1);
              }}
            >
              Words
            </DropdownMenu.RadioItem>
            <DropdownMenu.RadioItem
              value="grammar"
              onSelect={() => {
                setGrammar(true);
                setPage(1);
              }}
            >
              Grammar
            </DropdownMenu.RadioItem>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
        <span className="app-review-toolbar-hint">
          Drag a card or use the buttons below.
        </span>
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
          <Button
            className="app-review-skip"
            variant="ghost"
            onClick={() => reviewCardRef.current?.swipe("skip")}
          >
            <X size={17} /> Skip
          </Button>
          <button
            type="button"
            className="app-review-back"
            disabled={page <= 1}
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            aria-label="Previous card"
          >
            <ArrowLeft size={18} />
          </button>
          <Button
            className="app-review-know"
            onClick={() => reviewCardRef.current?.swipe("know")}
          >
            Know <ArrowRight size={17} />
          </Button>
        </div>
      )}
    </div>
  );
}
