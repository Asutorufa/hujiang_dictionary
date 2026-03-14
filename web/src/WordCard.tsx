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
    <Card className="break-inside-avoid mb-4 shadow-sm hover:shadow-md transition-shadow">
      <Flex justify="between" align="start" mb="2">
        <Flex direction="column" className="max-w-[70%]">
          <Text size="5" weight="bold" className="break-words">
            {word.word}
          </Text>
          <Text size="2" color="gray">
            {new Date(word.update_time * 1000).toLocaleDateString()}
          </Text>
        </Flex>

        <Flex align="center" gap="1">
          <DropdownMenu.Root>
            <DropdownMenu.Trigger disabled={loading}>
              <Badge
                size="1"
                variant="soft"
                color={
                  word.priority === 0
                    ? "green"
                    : word.priority === 1
                      ? "orange"
                      : "gray"
                }
                className="cursor-pointer"
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
                color="gray"
                onSelect={() => handlePriorityChange("2")}
              >
                High
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>

          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              <IconButton size="1" variant="ghost" color="gray">
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

      <Box pt="2">
        {word.example && (
          <Box className="bg-default-50 rounded-lg p-3 mb-2 text-small">
            <Markdown>{word.example}</Markdown>
          </Box>
        )}

        <Spoiler>
          <Markdown>{word.explain}</Markdown>
        </Spoiler>

        <Separator my="3" size="4" />

        <Flex justify="between" align="center">
          <Text size="1" color="gray">
            Review: {new Date(word.reminder_time * 1000).toLocaleDateString()}
          </Text>
          <Tooltip content="Increment Anki Count">
            <Button
              size="1"
              variant="ghost"
              color="gray"
              radius="full"
              onClick={handleIncrement}
              loading={isIncrementing}
            >
              {!isIncrementing && <Text size="2">+</Text>}
              {word.anki_count}
            </Button>
          </Tooltip>
        </Flex>
      </Box>
    </Card>
  );
}
