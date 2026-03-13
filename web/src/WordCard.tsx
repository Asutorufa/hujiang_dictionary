import {
  changePriority,
  EditIcon,
  getPriorityColor,
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
  CardBody,
  CardHeader,
  Chip,
  Divider,
  Dropdown,
  DropdownItem,
  DropdownMenu,
  DropdownTrigger,
  Tooltip,
} from "@heroui/react";
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
      <CardHeader className="flex justify-between items-start pb-0">
        <div className="flex flex-col max-w-[70%]">
          <h3 className="text-lg font-bold break-words">{word.word}</h3>
          <span className="text-tiny text-default-400">
            {new Date(word.update_time * 1000).toLocaleDateString()}
          </span>
        </div>

        <div className="flex items-center gap-1">
          <Dropdown isDisabled={loading}>
            <DropdownTrigger>
              <Chip
                size="sm"
                variant="flat"
                color={getPriorityColor(word.priority)}
                className={`cursor-pointer px-2 min-w-unit-12 ${loading ? "opacity-50 pointer-events-none" : ""}`}
              >
                {getPriorityText(word.priority)}
              </Chip>
            </DropdownTrigger>
            <DropdownMenu
              aria-label="Priority Actions"
              onAction={(key) => handlePriorityChange(key as string)}
            >
              <DropdownItem key="0" className="text-success">
                Low
              </DropdownItem>
              <DropdownItem key="1" className="text-warning">
                Medium
              </DropdownItem>
              <DropdownItem key="2" className="text-secondary">
                High
              </DropdownItem>
            </DropdownMenu>
          </Dropdown>

          <Dropdown>
            <DropdownTrigger>
              <Button
                isIconOnly
                size="sm"
                variant="light"
                className="min-w-unit-8 w-unit-8 h-unit-8"
              >
                <MoreVertIcon />
              </Button>
            </DropdownTrigger>
            <DropdownMenu aria-label="Card Actions">
              <DropdownItem
                key="edit"
                startContent={<EditIcon />}
                onPress={onEdit}
              >
                Edit
              </DropdownItem>
              <DropdownItem
                key="delete"
                className="text-danger"
                color="danger"
                startContent={<TrashIcon />}
                onPress={onDelete}
              >
                Delete
              </DropdownItem>
            </DropdownMenu>
          </Dropdown>
        </div>
      </CardHeader>

      <CardBody className="pt-2">
        {word.example && (
          <div className="bg-default-50 rounded-lg p-3 mb-2 text-small">
            <Markdown>{word.example}</Markdown>
          </div>
        )}

        <Spoiler>
          <Markdown>{word.explain}</Markdown>
        </Spoiler>

        <Divider className="my-3" />

        <div className="flex justify-between items-center">
          <div className="text-tiny text-default-400">
            Review: {new Date(word.reminder_time * 1000).toLocaleDateString()}
          </div>
          <Tooltip content="Increment Anki Count">
            <Button
              size="sm"
              variant="light"
              radius="full"
              className="text-tiny px-1 h-6 min-w-12 text-default-400 hover:text-primary"
              onPress={handleIncrement}
              startContent={
                !isIncrementing ? <span className="text-small">+</span> : null
              }
              isLoading={isIncrementing}
            >
              {word.anki_count}
            </Button>
          </Tooltip>
        </div>
      </CardBody>
    </Card>
  );
}
