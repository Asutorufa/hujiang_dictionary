import {
  BookIcon,
  ConfirmModal,
  countWord,
  deleteWord,
  FilterIcon,
  ListWordResponse,
  queryWord,
  RefreshIcon,
  SaveWordModal,
} from "@/components";
import WordCard from "@/WordCard";
import { DropdownMenu, Tooltip, IconButton, Flex } from "@radix-ui/themes";
import { useEffect, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import { LoadingOverlay } from "@/ui/LoadingOverlay";
import { EmptyState } from "@/ui/EmptyState";
import { PageContainer } from "@/ui/PageContainer";
import { PageHeader } from "@/ui/PageHeader";
import { Pager } from "@/ui/Pager";
import { BookOpen } from "lucide-react";

const PlusIcon = ({
  size = 24,
  width,
  height,
  ...props
}: {
  size?: number;
  width?: number;
  height?: number;
}) => {
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
      <g
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
      >
        <path d="M6 12h12" />
        <path d="M12 18V6" />
      </g>
    </svg>
  );
};

export default function Words() {
  const [words, setWords] = useState<ListWordResponse[]>([]);
  const [page, setPage] = useLocalStorage<number>("page", 1);
  const [total, setTotal] = useLocalStorage<number>("total_page", 100); // 100 is just default
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [removeWord, setRemoveWord] = useState("");
  const [refresh, setRefresh] = useState(0);
  const [newWord, setNewWord] = useState<{
    origin?: string;
    new: ListWordResponse;
  }>({
    new: {
      word: "",
      example: "",
      explain: "",
      add_time: 0,
      update_time: 0,
      reminder_time: 0,
      anki_count: 0,
      priority: 0,
      type: 0,
    },
  });
  const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");
  const [grammar, setGrammar] = useLocalStorage("grammar", false);

  // Note: The original code fetched 'count' to set 'total', but it seemed to rely on a separate effect.
  // I'll keep the logic simple: queryWord returns list. countWord returns size.
  // The original code had two useEffects.

  // Effect to get total count
  useEffect(() => {
    countWord(grammar, (size) => {
      if (size) {
        const totalPages = Math.ceil(size / 10);
        setTotal(totalPages || 1);
        if (page > totalPages && totalPages > 0) setPage(totalPages);
      }
    });
  }, [refresh, grammar, page, setPage, setTotal]);

  // Effect to get words
  useEffect(() => {
    setLoading(true);
    queryWord(page, 10, orderBy, grammar, (data) => {
      if (data) {
        setWords(data);
      }
      setLoading(false);
    });
  }, [page, orderBy, refresh, grammar]);

  const handleWordUpdate = (index: number, updatedWord: ListWordResponse) => {
    setWords((prev) => {
      const newWords = [...prev];
      newWords[index] = updatedWord;
      return newWords;
    });
  };

  return (
    <div className="app-page-shell min-h-dvh">
      <ConfirmModal
        title={`Are you sure you want to delete ${removeWord}?`}
        open={confirmOpen}
        color="danger"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onChange={(p) => setConfirmOpen(p)}
        onConfirm={async () => {
          if (removeWord) {
            await deleteWord(removeWord, (error) => {
              if (!error) {
                setRefresh(refresh + 1);
              }
            });
          }
        }}
      />

      <SaveWordModal
        open={open}
        onChange={(p) => setOpen(p)}
        word={newWord.new.word}
        explain={newWord.new.explain}
        example={newWord.new.example}
        type={newWord.new.type}
        origin={newWord.origin}
        onSaved={() => {
          setRefresh(refresh + 1);
        }}
      />

      <LoadingOverlay show={loading} />

      <PageContainer size="7xl" className="space-y-4">
        <PageHeader
          title="Words"
          density="compact"
          actions={
            <Flex gap="2" align="center" wrap="wrap">
              <Pager
                page={page}
                total={total || 1}
                onPageChange={(p) => setPage(p)}
              />

              <DropdownMenu.Root modal={false}>
                <DropdownMenu.Trigger>
                  <IconButton
                    variant="ghost"
                    color="gray"
                    className="app-icon-chip"
                  >
                    <FilterIcon size={18} />
                  </IconButton>
                </DropdownMenu.Trigger>
                <DropdownMenu.Content>
                  <DropdownMenu.Label>Sort By</DropdownMenu.Label>
                  <DropdownMenu.RadioGroup
                    value={orderBy}
                    onValueChange={setOrderBy}
                  >
                    <DropdownMenu.RadioItem value="word">
                      Word (A-Z)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="word desc">
                      Word (Z-A)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="priority">
                      Priority (Low-High)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="priority desc">
                      Priority (High-Low)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="add_time">
                      Date Added (Oldest)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="add_time desc">
                      Date Added (Newest)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="update_time">
                      Date Updated (Oldest)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="update_time desc">
                      Date Updated (Newest)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="anki_count">
                      Count (Low-High)
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="anki_count desc">
                      Count (High-Low)
                    </DropdownMenu.RadioItem>
                  </DropdownMenu.RadioGroup>

                  <DropdownMenu.Separator />

                  <DropdownMenu.Label>Filter</DropdownMenu.Label>
                  <DropdownMenu.RadioGroup
                    value={grammar ? "grammar" : "words"}
                    onValueChange={(v) => setGrammar(v === "grammar")}
                  >
                    <DropdownMenu.RadioItem value="words">
                      Words
                    </DropdownMenu.RadioItem>
                    <DropdownMenu.RadioItem value="grammar">
                      <Flex gap="2" align="center">
                        <BookIcon size={18} />
                        Grammar
                      </Flex>
                    </DropdownMenu.RadioItem>
                  </DropdownMenu.RadioGroup>
                </DropdownMenu.Content>
              </DropdownMenu.Root>

              <Tooltip content="Add Word">
                <IconButton
                  size="1"
                  color="gray"
                  variant="solid"
                  className="app-icon-primary"
                  onClick={() => {
                    setNewWord({
                      new: {
                        word: "",
                        example: "",
                        explain: "",
                        add_time: 0,
                        update_time: 0,
                        reminder_time: 0,
                        anki_count: 0,
                        priority: 0,
                        type: 0,
                      },
                    });
                    setOpen(true);
                  }}
                >
                  <PlusIcon size={16} />
                </IconButton>
              </Tooltip>

              <Tooltip content="Refresh">
                <IconButton
                  size="1"
                  variant="ghost"
                  color="gray"
                  className="app-icon-chip"
                  onClick={() => setRefresh((r) => r + 1)}
                >
                  <RefreshIcon size={16} />
                </IconButton>
              </Tooltip>
            </Flex>
          }
        />

        {/* Masonry Layout using CSS columns */}
        <div className="columns-1 lg:columns-2 2xl:columns-3 gap-4">
          {words
            .filter((w) => w.word && w.word.length > 0)
            .map((w, i) => (
              <div key={w.word + i} className="break-inside-avoid mb-4">
                <WordCard
                  word={w}
                  onEdit={() => {
                    setNewWord({
                      origin: w.word,
                      new: w,
                    });
                    setOpen(true);
                  }}
                  onDelete={() => {
                    setRemoveWord(w.word);
                    setConfirmOpen(true);
                  }}
                  onWordUpdate={(updated) => handleWordUpdate(i, updated)}
                />
              </div>
            ))}

          {!loading && words.length === 0 && (
            <div className="col-span-full">
              <EmptyState
                title="No words found"
                description="Add your first word to start reviewing."
                icon={<BookOpen size={28} className="text-[var(--gray-a11)]" />}
                actionLabel="Add a word"
                onAction={() => setOpen(true)}
              />
            </div>
          )}
        </div>
      </PageContainer>
    </div>
  );
}
