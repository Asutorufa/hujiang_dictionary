import {
  changePriority,
  EditIcon,
  getPriorityText,
  incrementRemindCount,
  ListWordResponse,
  Markdown,
  MoreVertIcon,
  Spoiler,
  TrashIcon,
} from "@/components";
import { Badge, Button, DropdownMenu, Tooltip } from "@radix-ui/themes";
import { BookOpenCheck } from "lucide-react";
import { useState } from "react";

interface WordCardProps {
  word: ListWordResponse;
  onEdit: () => void;
  onDelete: () => void;
  onWordUpdate: (updatedWord: ListWordResponse) => void;
}

export default function WordCard({ word, onEdit, onDelete, onWordUpdate }: WordCardProps) {
  const [loading, setLoading] = useState(false);
  const [isIncrementing, setIsIncrementing] = useState(false);

  const handlePriorityChange = async (key: string) => {
    const priority = Number.parseInt(key, 10);
    if (priority === word.priority) return;
    setLoading(true);
    await changePriority(word.word, priority, (error) => {
      setLoading(false);
      if (!error) onWordUpdate({ ...word, priority });
    });
  };

  const handleIncrement = async () => {
    if (isIncrementing) return;
    setIsIncrementing(true);
    await incrementRemindCount(word.word, (error) => {
      setIsIncrementing(false);
      if (!error) onWordUpdate({ ...word, anki_count: word.anki_count + 1 });
    });
  };

  return (
    <article className={`app-word-card app-word-card-priority-${word.priority}`}>
      <div className="app-word-card-main">
        <header className="app-word-card-header">
          <div className="app-word-card-title">
            <h2>{word.word}</h2>
            <span>Updated {new Date(word.update_time * 1000).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" })}</span>
          </div>
          <div className="app-word-card-actions">
            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger disabled={loading}>
                <Badge
                  size="1"
                  variant="soft"
                  color={word.priority === 0 ? "green" : word.priority === 1 ? "orange" : "red"}
                >
                  {getPriorityText(word.priority)}
                </Badge>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Item color="green" onSelect={() => handlePriorityChange("0")}>Low</DropdownMenu.Item>
                <DropdownMenu.Item color="orange" onSelect={() => handlePriorityChange("1")}>Medium</DropdownMenu.Item>
                <DropdownMenu.Item color="red" onSelect={() => handlePriorityChange("2")}>High</DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger>
                <button type="button" className="app-word-more" aria-label={`More actions for ${word.word}`}>
                  <MoreVertIcon />
                </button>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Item onSelect={onEdit}><EditIcon /> Edit word</DropdownMenu.Item>
                <DropdownMenu.Item color="red" onSelect={onDelete}><TrashIcon /> Delete word</DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </div>
        </header>

        <div className="app-word-card-content">
          {word.example && (
            <section className="app-word-example">
              <div className="app-word-section-label">Example</div>
              <div className="prose"><Markdown>{word.example}</Markdown></div>
            </section>
          )}

          <section className={`app-word-meaning${word.example ? "" : " app-word-meaning-full"}`}>
            <div className="app-word-section-label">Meaning</div>
            <Spoiler>
              <div className="prose app-word-meaning-content"><Markdown>{word.explain}</Markdown></div>
            </Spoiler>
          </section>
        </div>

        <footer className="app-word-card-footer">
          <div className="app-word-card-footer-meta">
            <span>Next review</span>
            <strong>{new Date(word.reminder_time * 1000).toLocaleDateString()}</strong>
          </div>
          <Tooltip content="Mark as reviewed">
            <Button size="1" variant="ghost" className="app-word-review-button" onClick={handleIncrement} loading={isIncrementing}>
              {!isIncrementing && <BookOpenCheck size={14} />}
              <span>Review</span>
              <strong>{word.anki_count}</strong>
            </Button>
          </Tooltip>
        </footer>
      </div>
    </article>
  );
}
