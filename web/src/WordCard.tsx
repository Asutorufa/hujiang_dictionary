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
import {
  Button,
  Card,
  Badge,
  Separator,
  DropdownMenu,
  Tooltip,
  IconButton,
  Flex,
  Text,
  Box,
} from "@radix-ui/themes";
import { useState } from "react";

interface WordCardProps {
  word: ListWordResponse;
  onEdit: () => void;
  onDelete: () => void;
  onWordUpdate: (updatedWord: ListWordResponse) => void;
}

export default function WordCard({
  word,
  onEdit,
  onDelete,
  onWordUpdate,
}: WordCardProps) {
  const [loading, setLoading] = useState(false);
  const [isIncrementing, setIsIncrementing] = useState(false);

  const handlePriorityChange = async (key: string) => {
    const priority = parseInt(key);
    if (priority === word.priority) return;
    setLoading(true);
    await changePriority(word.word, priority, (error) => {
      setLoading(false);
      if (!error) {
        onWordUpdate({ ...word, priority });
      }
    });
  };

  const handleIncrement = async () => {
    if (isIncrementing) return;
    setIsIncrementing(true);
    await incrementRemindCount(word.word, (error) => {
      setIsIncrementing(false);
      if (!error) {
        onWordUpdate({ ...word, anki_count: word.anki_count + 1 });
      }
    });
  };

  return (
    <div className="w-full">
      <Card className="app-section-card">
        <Flex justify="between" align="start" gap="3">
          <Flex direction="column" className="max-w-[70%]">
            <Text size="5" weight="bold" className="break-words">
              {word.word}
            </Text>
            <Text size="1" color="gray">
              {new Date(word.update_time * 1000).toLocaleDateString(undefined, {
                year: "numeric",
                month: "short",
                day: "numeric",
              })}
            </Text>
          </Flex>

          <Flex align="center" gap="2">
            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger
                disabled={loading}
                className="cursor-pointer"
              >
                <Badge
                  size="1"
                  variant="soft"
                  color={
                    word.priority === 0
                      ? "green"
                      : word.priority === 1
                        ? "orange"
                        : "red"
                  }
                  className="cursor-pointer"
                  style={{ cursor: "pointer" }}
                >
                  {getPriorityText(word.priority)}
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
                  color="red"
                  onSelect={() => handlePriorityChange("2")}
                >
                  High
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>

            <DropdownMenu.Root modal={false}>
              <DropdownMenu.Trigger className="cursor-pointer">
                <IconButton
                  size="1"
                  variant="ghost"
                  color="gray"
                  className="cursor-pointer"
                >
                  <MoreVertIcon />
                </IconButton>
              </DropdownMenu.Trigger>
              <DropdownMenu.Content>
                <DropdownMenu.Item onSelect={onEdit}>
                  <Flex gap="2" align="center">
                    <EditIcon /> Edit
                  </Flex>
                </DropdownMenu.Item>
                <DropdownMenu.Item color="red" onSelect={onDelete}>
                  <Flex gap="2" align="center">
                    <TrashIcon /> Delete
                  </Flex>
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </Flex>
        </Flex>

        <Box mt="3">
          {word.example && (
            <Box mb="3" p="3" className="app-muted-panel">
              <Text
                size="1"
                weight="bold"
                color="gray"
                className="block mb-1 uppercase tracking-widest"
              >
                Example
              </Text>
              <div className="prose prose-sm dark:prose-invert max-w-none italic opacity-80 max-h-32 overflow-y-auto custom-scrollbar pr-1">
                <Markdown>{word.example}</Markdown>
              </div>
            </Box>
          )}

          <Spoiler>
            <div className="prose prose-sm dark:prose-invert max-w-none max-h-64 overflow-y-auto custom-scrollbar pr-1">
              <Markdown>{word.explain}</Markdown>
            </div>
          </Spoiler>

          <Separator my="3" size="4" />

          <Flex justify="between" align="center">
            <Text size="1" color="gray">
              Review: {new Date(word.reminder_time * 1000).toLocaleDateString()}
            </Text>
            <Tooltip content="Learned times">
              <Button
                size="1"
                variant="soft"
                color="gray"
                radius="full"
                className="app-icon-chip"
                onClick={handleIncrement}
                loading={isIncrementing}
              >
                {!isIncrementing && (
                  <Text size="1" weight="bold">
                    +1
                  </Text>
                )}
                {!isIncrementing && (
                  <span
                    aria-hidden="true"
                    className="mx-1 h-3.5 w-px bg-[color-mix(in_srgb,var(--app-border-strong)_80%,transparent)]"
                  />
                )}
                <Text size="2" weight="bold">
                  {word.anki_count}
                </Text>
              </Button>
            </Tooltip>
          </Flex>
        </Box>
      </Card>
    </div>
  );
}
