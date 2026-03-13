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
import {
  Button,
  Dropdown,
  DropdownItem,
  DropdownMenu,
  DropdownSection,
  DropdownTrigger,
  Pagination,
  Spinner,
  Tooltip,
} from "@heroui/react";
import { useEffect, useState } from "react";
import { useLocalStorage } from "usehooks-ts";

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
    <>
      <ConfirmModal
        title={`Are you sure you want to delete ${removeWord}?`}
        open={confirmOpen}
        color="danger"
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

      {loading && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-background/20 backdrop-blur-sm">
          <Spinner size="lg" />
        </div>
      )}

      <div className="container mx-auto p-2 sm:p-4 max-w-7xl">
        {/* Sticky Header */}
        <div className="sticky top-4 z-40 bg-background/60 backdrop-blur-xl rounded-2xl shadow-lg border border-default-200/50 p-3 mb-8 flex flex-wrap items-center justify-between gap-4 transition-all hover:shadow-xl">
          <Pagination
            isCompact
            showControls
            total={total}
            page={page}
            onChange={setPage}
            className="overflow-visible"
            classNames={{
              wrapper: "shadow-none bg-transparent gap-1",
              item: "bg-transparent border-small border-default-200 shadow-none hover:bg-default-100",
              cursor: "bg-foreground text-background font-bold",
            }}
          />

          <div className="flex gap-2 items-center ml-auto">
            <Dropdown>
              <DropdownTrigger>
                <Button
                  variant="flat"
                  startContent={<FilterIcon />}
                  className="bg-default-100 font-medium"
                >
                  View Options
                </Button>
              </DropdownTrigger>
              <DropdownMenu
                aria-label="View Options"
                closeOnSelect={false}
                selectionMode="single"
                selectedKeys={[orderBy]}
                onSelectionChange={(keys) => {
                  const selected = Array.from(keys)[0] as string;
                  if (selected) setOrderBy(selected);
                }}
              >
                <DropdownSection title="Sort By" showDivider>
                  <DropdownItem key="word">Word (A-Z)</DropdownItem>
                  <DropdownItem key="word desc">Word (Z-A)</DropdownItem>
                  <DropdownItem key="priority">
                    Priority (Low-High)
                  </DropdownItem>
                  <DropdownItem key="priority desc">
                    Priority (High-Low)
                  </DropdownItem>
                  <DropdownItem key="add_time">
                    Date Added (Oldest)
                  </DropdownItem>
                  <DropdownItem key="add_time desc">
                    Date Added (Newest)
                  </DropdownItem>
                  <DropdownItem key="update_time">
                    Date Updated (Oldest)
                  </DropdownItem>
                  <DropdownItem key="update_time desc">
                    Date Updated (Newest)
                  </DropdownItem>
                  <DropdownItem key="anki_count">Count (Low-High)</DropdownItem>
                  <DropdownItem key="anki_count desc">
                    Count (High-Low)
                  </DropdownItem>
                </DropdownSection>

                <DropdownSection title="Filter">
                  <DropdownItem
                    key="grammar-toggle"
                    startContent={<BookIcon size={18} />}
                    className={grammar ? "bg-primary-50 text-primary" : ""}
                    onPress={() => setGrammar(!grammar)}
                  >
                    {grammar ? "Show All Words" : "Show Grammar Only"}
                  </DropdownItem>
                </DropdownSection>
              </DropdownMenu>
            </Dropdown>

            <div className="h-6 w-[1px] bg-default-200 mx-1" />

            <Tooltip content="Add Word">
              <Button
                isIconOnly
                color="primary"
                variant="shadow"
                onPress={() => {
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
                <PlusIcon />
              </Button>
            </Tooltip>

            <Tooltip content="Refresh">
              <Button
                isIconOnly
                variant="light"
                className="text-default-500"
                onPress={() => setRefresh((r) => r + 1)}
              >
                <RefreshIcon />
              </Button>
            </Tooltip>
          </div>
        </div>

        {/* Masonry Grid Layout */}
        <div className="columns-1 sm:columns-2 lg:columns-3 xl:columns-4 gap-4 space-y-4 pb-20">
          {words
            .filter((w) => w.word && w.word.length > 0)
            .map((w, i) => (
              <div key={w.word + i} className="break-inside-avoid">
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
            <div className="flex flex-col items-center justify-center p-10 text-default-400 col-span-full w-full">
              <p>No words found.</p>
              <Button
                variant="light"
                color="primary"
                className="mt-2"
                onPress={() => setOpen(true)}
              >
                Add your first word
              </Button>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
