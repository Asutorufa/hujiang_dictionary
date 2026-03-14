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
  DropdownMenu,
  Spinner,
  Tooltip,
  IconButton,
  Flex,
  Text,
} from "@radix-ui/themes";
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
          <Spinner size="3" />
        </div>
      )}

      <div className="container mx-auto p-2 sm:p-4 max-w-7xl">
        {/* Sticky Header */}
        <div className="sticky top-4 z-40 bg-background/60 backdrop-blur-xl rounded-2xl shadow-lg border border-default-200/50 p-3 mb-8 flex flex-wrap items-center justify-between gap-4 transition-all hover:shadow-xl">
          <Flex gap="2" align="center">
            <IconButton
              variant="soft"
              color="gray"
              disabled={page <= 1}
              onClick={() => setPage((p) => Math.max(1, p - 1))}
            >
              <svg
                width="15"
                height="15"
                viewBox="0 0 15 15"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M8.84182 3.13514C9.04327 3.32401 9.05348 3.64042 8.86462 3.84188L5.43521 7.49991L8.86462 11.1579C9.05348 11.3594 9.04327 11.6758 8.84182 11.8647C8.64036 12.0535 8.32394 12.0433 8.13508 11.8419L4.38508 7.84188C4.20477 7.64955 4.20477 7.35027 4.38508 7.15794L8.13508 3.15794C8.32394 2.95648 8.64036 2.94628 8.84182 3.13514Z"
                  fill="currentColor"
                  fillRule="evenodd"
                  clipRule="evenodd"
                ></path>
              </svg>
            </IconButton>
            <Text size="2" weight="medium">
              {page} / {total || 1}
            </Text>
            <IconButton
              variant="soft"
              color="gray"
              disabled={page >= total}
              onClick={() => setPage((p) => Math.min(total, p + 1))}
            >
              <svg
                width="15"
                height="15"
                viewBox="0 0 15 15"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M6.1584 3.13508C5.95694 3.32394 5.94673 3.64036 6.13559 3.84182L9.565 7.49991L6.13559 11.158C5.94673 11.3595 5.95694 11.6759 6.1584 11.8648C6.35986 12.0536 6.67628 12.0434 6.86514 11.842L10.6151 7.84197C10.7954 7.64964 10.7954 7.35036 10.6151 7.15803L6.86514 3.15803C6.67628 2.95657 6.35986 2.94637 6.1584 3.13508Z"
                  fill="currentColor"
                  fillRule="evenodd"
                  clipRule="evenodd"
                ></path>
              </svg>
            </IconButton>
          </Flex>

          <div className="flex gap-2 items-center ml-auto">
            <DropdownMenu.Root>
              <DropdownMenu.Trigger>
                <Button
                  variant="soft"
                  color="gray"
                  className="bg-default-100 font-medium"
                >
                  <FilterIcon /> View Options
                </Button>
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
                <DropdownMenu.CheckboxItem
                  checked={grammar}
                  onCheckedChange={(checked) => setGrammar(checked)}
                >
                  <Flex gap="2" align="center">
                    <BookIcon size={18} />
                    {grammar ? "Show Grammar Only" : "Show All Words"}
                  </Flex>
                </DropdownMenu.CheckboxItem>
              </DropdownMenu.Content>
            </DropdownMenu.Root>

            <div className="h-6 w-[1px] bg-default-200 mx-1" />

            <Tooltip content="Add Word">
              <IconButton
                color="blue"
                variant="solid"
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
                <PlusIcon />
              </IconButton>
            </Tooltip>

            <Tooltip content="Refresh">
              <IconButton
                variant="ghost"
                color="gray"
                onClick={() => setRefresh((r) => r + 1)}
              >
                <RefreshIcon />
              </IconButton>
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
                variant="ghost"
                color="blue"
                className="mt-2"
                onClick={() => setOpen(true)}
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
