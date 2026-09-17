import {
  ConfirmModal,
  countWord,
  deleteWord,
  ListWordResponse,
  queryWord,
  SaveWordModal,
} from "@/components";
import WordCard from "@/WordCard";
import { DropdownMenu } from "@radix-ui/themes";
import { useEffect, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import { LoadingOverlay } from "@/ui/LoadingOverlay";
import { EmptyState } from "@/ui/EmptyState";
import { PageContainer } from "@/ui/PageContainer";
import { PageHeader } from "@/ui/PageHeader";
import { Pager } from "@/ui/Pager";
import {
  BookOpen,
  RefreshCw,
  Search,
  SlidersHorizontal,
  Plus,
} from "lucide-react";

const createEmptyWord = (): ListWordResponse => ({
  word: "",
  example: "",
  explain: "",
  add_time: 0,
  update_time: 0,
  reminder_time: 0,
  anki_count: 0,
  priority: 0,
  type: 0,
});

export default function Words() {
  const [words, setWords] = useState<ListWordResponse[]>([]);
  const [page, setPage] = useLocalStorage<number>("page", 1);
  const [total, setTotal] = useLocalStorage<number>("words_total_pages", 1);
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [removeWord, setRemoveWord] = useState("");
  const [refresh, setRefresh] = useState(0);
  const [search, setSearch] = useState("");
  const [newWord, setNewWord] = useState<{
    origin?: string;
    new: ListWordResponse;
  }>({
    new: createEmptyWord(),
  });
  const [orderBy, setOrderBy] = useLocalStorage("order_by", "word");
  const [grammar, setGrammar] = useLocalStorage("grammar", false);

  useEffect(() => {
    countWord(grammar, (size) => {
      if (size !== undefined) {
        const totalPages = Math.ceil(size / 10);
        const safeTotalPages = Math.max(totalPages, 1);
        setTotal(safeTotalPages);
        if (page > safeTotalPages) setPage(safeTotalPages);
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

  const openNewWord = () => {
    setNewWord({ new: createEmptyWord() });
    setOpen(true);
  };

  const handleWordUpdate = (index: number, updatedWord: ListWordResponse) => {
    setWords((prev) => {
      const newWords = [...prev];
      newWords[index] = updatedWord;
      return newWords;
    });
  };

  const normalizedSearch = search.trim().toLocaleLowerCase();
  const visibleWords = words
    .map((word, index) => ({ word, index }))
    .filter(({ word }) => {
      return (
        word.word &&
        word.word.length > 0 &&
        (!normalizedSearch ||
          `${word.word} ${word.example} ${word.explain}`
            .toLocaleLowerCase()
            .includes(normalizedSearch))
      );
    });

  return (
    <div className="app-library-page">
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
                setRefresh((value) => value + 1);
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
          setRefresh((value) => value + 1);
        }}
      />

      <LoadingOverlay show={loading} />

      <PageContainer size="7xl">
        <PageHeader
          title="Vocabulary"
          subtitle="Save, review, and keep the words you want to remember close at hand."
          actions={
            <button type="button" className="app-primary-action" onClick={openNewWord}>
              <Plus size={17} />
              Add word
            </button>
          }
        />

        <div className="app-library-toolbar">
          <label className="app-search-field">
            <Search size={17} aria-hidden="true" />
            <input
              type="search"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Find a saved word"
              aria-label="Find a saved word"
            />
          </label>
          <div className="app-library-toolbar-actions">
            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger>
                <button type="button" className="app-library-control">
                  <SlidersHorizontal size={16} />
                  <span>Sort & display</span>
                </button>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Label>Sort by</DropdownMenu.Label>
                <DropdownMenu.RadioGroup
                  value={orderBy}
                  onValueChange={setOrderBy}
                >
                  <DropdownMenu.RadioItem value="word">
                    Word (A–Z)
                  </DropdownMenu.RadioItem>
                  <DropdownMenu.RadioItem value="word desc">
                    Word (Z–A)
                  </DropdownMenu.RadioItem>
                  <DropdownMenu.RadioItem value="priority desc">
                    Priority
                  </DropdownMenu.RadioItem>
                  <DropdownMenu.RadioItem value="add_time desc">
                    Recently added
                  </DropdownMenu.RadioItem>
                  <DropdownMenu.RadioItem value="anki_count desc">
                    Most reviewed
                  </DropdownMenu.RadioItem>
                </DropdownMenu.RadioGroup>
                <DropdownMenu.Separator />
                <DropdownMenu.Label>Show</DropdownMenu.Label>
                <DropdownMenu.RadioGroup
                  value={grammar ? "grammar" : "words"}
                  onValueChange={(v) => setGrammar(v === "grammar")}
                >
                  <DropdownMenu.RadioItem value="words">
                    Words
                  </DropdownMenu.RadioItem>
                  <DropdownMenu.RadioItem value="grammar">
                    Grammar
                  </DropdownMenu.RadioItem>
                </DropdownMenu.RadioGroup>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
            <div className="app-library-page-count">
              <Pager
                page={page}
                total={total || 1}
                onPageChange={(p) => setPage(p)}
              />
            </div>
            <button
              type="button"
              className="app-library-icon-button"
              onClick={() => setRefresh((r) => r + 1)}
              aria-label="Refresh vocabulary"
            >
              <RefreshCw size={17} />
            </button>
          </div>
        </div>

        <div className="app-library-list-heading">
          <div>
            <h2>{grammar ? "Grammar notes" : "Saved vocabulary"}</h2>
            <span>
              Review, edit, or adjust a word without leaving the list.
            </span>
          </div>
          <span>
            {visibleWords.length} {normalizedSearch ? "matching" : "shown"}
          </span>
        </div>

        <div className="app-word-grid">
          {visibleWords.map(({ word: w, index }) => (
            <div key={w.word + index} className="app-word-grid-item">
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
                onWordUpdate={(updated) => handleWordUpdate(index, updated)}
              />
            </div>
          ))}

          {!loading && visibleWords.length === 0 && (
            <div className="col-span-full">
              <EmptyState
                title={normalizedSearch ? "No matching words" : "No words found"}
                description={
                  normalizedSearch
                    ? "Try a different search or clear the search field."
                    : "Add your first word to start reviewing."
                }
                icon={<BookOpen size={28} className="text-[var(--gray-a11)]" />}
                actionLabel={normalizedSearch ? "Clear search" : "Add a word"}
                onAction={normalizedSearch ? () => setSearch("") : openNewWord}
              />
            </div>
          )}
        </div>
      </PageContainer>
    </div>
  );
  /*
  return (
    <div className="app-page-shell min-h-dvh">
  */
}
