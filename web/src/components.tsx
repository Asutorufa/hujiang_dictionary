import { authorizedRequest } from "@/lib/api";
import { Button, Dialog, Switch, TextArea, Flex, Text, Box } from "@radix-ui/themes";
import { FC, ReactNode, useEffect, useState } from "react";
import { toast } from "sonner";
import { motion, AnimatePresence } from "framer-motion";

export function useDisclosure(initialState = false) {
  const [isOpen, setIsOpen] = useState(initialState);
  const onOpen = () => setIsOpen(true);
  const onClose = () => setIsOpen(false);
  const onOpenChange = (open?: boolean) => {
    setIsOpen(open !== undefined ? open : !isOpen);
  };
  return { isOpen, onOpen, onClose, onOpenChange };
}

export const addToast = ({
  title,
  description,
  color,
  timeout,
}: {
  title: string;
  description?: string;
  color?: "success" | "danger" | "warning" | "default";
  timeout?: number;
}) => {
  const options: Record<string, unknown> = { description };
  if (timeout !== undefined) {
    options.duration = timeout === 0 ? Infinity : timeout;
  }

  if (color === "danger") {
    toast.error(title, options);
  } else if (color === "success") {
    toast.success(title, options);
  } else if (color === "warning") {
    toast.warning(title, options);
  } else {
    toast(title, options);
  }
};

export const SaveWordModal: FC<{
  open: boolean;
  onChange: (open: boolean) => void;
  origin?: string;
  word?: string;
  explain?: string;
  example?: string;
  type: number;
  onSaved?: () => void;
}> = ({ open, onChange, origin, word, explain, type, onSaved, example }) => {
  const [saving, setSaving] = useState(false);
  const { isOpen, onOpenChange } = useDisclosure();

  useEffect(() => {
    onOpenChange(open);
  }, [open, onOpenChange]);

  const [newWord, setNewWord] = useState(word || "");
  const [newExplain, setNewExplain] = useState(explain || "");
  const [newExample, setNewExample] = useState(example || "");
  const [newType, setNewType] = useState(type);

  useEffect(() => {
    setNewWord(word || "");
    setNewExplain(explain || "");
    setNewExample(example || "");
    setNewType(type);
  }, [word, explain, type, example]);

  const handleOpenChange = (open: boolean) => {
    onChange(open);
    onOpenChange(open);
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={handleOpenChange}>
      <Dialog.Content maxWidth="500px">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ type: "spring", duration: 0.4, bounce: 0.2 }}
        >
          <Dialog.Title>Save to Vocabulary</Dialog.Title>
          <Flex direction="column" gap="4" mt="4">
            <Box>
              <Text as="label" size="2" weight="medium">
                <Flex gap="2" align="center" className="cursor-pointer">
                  <Switch
                    checked={newType === 1}
                    onCheckedChange={(p) => setNewType(p ? 1 : 0)}
                  />
                  Grammar / Pattern
                </Flex>
              </Text>
            </Box>

            <Flex direction="column" gap="1">
              <Text as="label" size="2" weight="bold" color="gray">
                Word / Phrase
              </Text>
              <TextArea
                color={newWord.length === 0 ? "red" : undefined}
                value={newWord}
                onChange={(p) => setNewWord(p.target.value)}
                placeholder="What did you learn?"
                variant="surface"
              />
            </Flex>

            <Flex direction="column" gap="1">
              <Text as="label" size="2" weight="bold" color="gray">
                Meaning / Explanation
              </Text>
              <TextArea
                color={newExplain.length === 0 ? "red" : undefined}
                value={newExplain}
                onChange={(p) => setNewExplain(p.target.value)}
                placeholder="Explain it here..."
                variant="surface"
                className="min-h-[100px]"
              />
            </Flex>

            <Flex direction="column" gap="1">
              <Text as="label" size="2" weight="bold" color="gray">
                Context / Example
              </Text>
              <TextArea
                value={newExample}
                onChange={(p) => setNewExample(p.target.value)}
                placeholder="Add an example sentence..."
                variant="surface"
                className="min-h-[80px]"
              />
            </Flex>
          </Flex>
          <Flex gap="3" mt="5" justify="end">
            <Button
              variant="soft"
              color="gray"
              className="cursor-pointer"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button
              className="cursor-pointer"
              loading={saving}
              onClick={() => {
                if (newWord.length === 0 || newExplain.length === 0) return;
                setSaving(true);
                saveWord(
                  newWord,
                  newExplain,
                  newExample,
                  newType,
                  (error) => {
                    setSaving(true);
                    handleOpenChange(false);
                    if (!error && onSaved) onSaved();
                  },
                  origin,
                );
              }}
            >
              Save Word
            </Button>
          </Flex>
        </motion.div>
      </Dialog.Content>
    </Dialog.Root>
  );
};


export type ListWordResponse = {
  word: string;
  explain: string;
  example: string;
  add_time: number;
  update_time: number;
  reminder_time: number;
  anki_count: number;
  priority: number;
  type: number;
};

export async function wordRequest<T>(
  path: string,
  body: string,
  callback: (data?: T, error?: string) => void,
) {
  try {
    const res = await authorizedRequest(path, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: body,
    });

    if (!res.ok) {
      const errorText = await res.text();
      throw new Error(`Request failed with status ${res.status}: ${errorText}`);
    }

    const data = (await res.json()) as T;
    callback(data, undefined);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    callback(undefined, errorMessage);
    addToast({
      title: `Words Request Error(${path})`,
      description: errorMessage,
      color: "danger",
      timeout: 0,
    });
  }
}

export async function queryWord(
  page: number,
  size: number,
  order_by: string,
  grammar: boolean,
  callback: (data?: ListWordResponse[], error?: string) => void,
) {
  await wordRequest<ListWordResponse[]>(
    "/word/list",
    JSON.stringify({
      page_size: size,
      page_number: page,
      order_by: order_by,
      type: grammar ? 1 : 0,
    }),
    callback,
  );
}

type CountWordResponse = {
  size: number;
};

export async function countWord(
  grammar: boolean,
  callback: (size?: number, error?: string) => void,
) {
  await wordRequest<CountWordResponse>(
    "/word/count",
    JSON.stringify({
      type: grammar ? 1 : 0,
    }),
    (data, error) => {
      callback(data?.size, error);
    },
  );
}

export async function saveWord(
  word: string,
  explain: string,
  example: string,
  type: number,
  callback: (error?: string) => void,
  originalWord?: string,
) {
  await wordRequest<object>(
    "/word/save",
    JSON.stringify({
      origin: originalWord,
      word: word,
      explain: explain,
      example: example,
      type: type,
    }),
    (_, error) => {
      callback(error);
    },
  );
}

export async function deleteWord(
  word: string,
  callback: (error?: string) => void,
) {
  await wordRequest<object>(
    "/word/delete",
    JSON.stringify({
      word: word,
    }),
    (_, error) => {
      callback(error);
    },
  );
}

export async function incrementRemindCount(
  word: string,
  callback: (error?: string) => void,
) {
  await wordRequest<object>(
    "/word/remind_count_increment",
    JSON.stringify({
      word: word,
    }),
    (_, error) => {
      callback(error);
    },
  );
}

export async function changePriority(
  word: string,
  priority: number,
  callback: (error?: string) => void,
) {
  await wordRequest<object>(
    "/word/priority",
    JSON.stringify({
      word: word,
      priority: priority,
    }),
    (_, error) => {
      callback(error);
    },
  );
}

type ModelResponse = {
  name: string;
  models: string[];
};

export async function listModel(
  callback: (data?: ModelResponse[], error?: string) => void,
) {
  await wordRequest<ModelResponse[]>("/word/ai_custom", "", (data, error) => {
    callback(data, error);
  });
}

type LegacyColor =
  | "danger"
  | "default"
  | "primary"
  | "secondary"
  | "success"
  | "warning";
type RadixColor =
  | "crimson"
  | "ruby"
  | "tomato"
  | "red"
  | "purple"
  | "violet"
  | "iris"
  | "indigo"
  | "blue"
  | "cyan"
  | "teal"
  | "jade"
  | "green"
  | "grass"
  | "brown"
  | "orange"
  | "sky"
  | "mint"
  | "lime"
  | "yellow"
  | "amber"
  | "gold"
  | "bronze"
  | "gray";

export const ConfirmModal: FC<{
  title: string;
  open: boolean;
  onChange: (open: boolean) => void;
  onConfirm: () => Promise<void>;
  color?: RadixColor | LegacyColor;
}> = ({ title, open, onConfirm, onChange, color }) => {
  const [loading, setLoading] = useState(false);

  // Map legacy HeroUI colors to Radix colors, otherwise pass through the given Radix color
  let radixColor: RadixColor | undefined;

  if (color === "danger") radixColor = "red";
  else if (color === "success") radixColor = "green";
  else if (color === "warning") radixColor = "orange";
  else if (color === "secondary" || color === "default") radixColor = "gray";
  else if (color === "primary") radixColor = "blue";
  else radixColor = color as RadixColor | undefined;

  return (
    <Dialog.Root open={open} onOpenChange={onChange}>
      <Dialog.Content maxWidth="400px">
        <Dialog.Title className="text-center mb-4">{title}</Dialog.Title>
        <Flex gap="3" mt="4" justify="center">
          <Button variant="soft" color="gray" onClick={() => onChange(false)}>
            Close
          </Button>
          <Button
            loading={loading}
            color={radixColor}
            onClick={async () => {
              setLoading(true);
              await onConfirm();
              setLoading(false);
              onChange(false);
            }}
          >
            Ok
          </Button>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
};

export type IconProps = {
  size?: number;
  width?: number;
  height?: number;
  className?: string;
};

export function FilterIcon({ size = 24, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="currentColor"
        d="M12 13.125a.75.75 0 0 1 .538 1.272l-4 4.125a.75.75 0 0 1-1.076 0l-4-4.125A.75.75 0 0 1 4 13.125h3.25V6a.75.75 0 1 1 1.5 0v7.125z"
      />
      <path
        fill="currentColor"
        d="M20 10.875a.75.75 0 0 0 .538-1.272l-4-4.125a.75.75 0 0 0-1.076 0l-4 4.125A.75.75 0 0 0 12 10.875h3.25V18a.75.75 0 0 0 1.5 0v-7.125z"
      />
    </svg>
  );
}

export function RefreshIcon({ size = 24, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="currentColor"
        d="M12.079 2.25c-4.794 0-8.734 3.663-9.118 8.333H2a.75.75 0 0 0-.528 1.283l1.68 1.666a.75.75 0 0 0 1.056 0l1.68-1.666a.75.75 0 0 0-.528-1.283h-.893c.38-3.831 3.638-6.833 7.612-6.833a7.66 7.66 0 0 1 6.537 3.643a.75.75 0 1 0 1.277-.786A9.16 9.16 0 0 0 12.08 2.25m8.761 8.217a.75.75 0 0 0-1.054 0L18.1 12.133a.75.75 0 0 0 .527 1.284h.899c-.382 3.83-3.651 6.833-7.644 6.833a7.7 7.7 0 0 1-6.565-3.644a.75.75 0 1 0-1.277.788a9.2 9.2 0 0 0 7.842 4.356c4.808 0 8.765-3.66 9.15-8.333H22a.75.75 0 0 0 .527-1.284z"
      />
    </svg>
  );
}

export function EditIcon({ size = 17, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="currentColor"
        fillRule="evenodd"
        d="M3.25 22a.75.75 0 0 1 .75-.75h16a.75.75 0 0 1 0 1.5H4a.75.75 0 0 1-.75-.75"
        clipRule="evenodd"
      />
      <path
        fill="currentColor"
        d="m11.52 14.929l5.917-5.917a8.2 8.2 0 0 1-2.661-1.787a8.2 8.2 0 0 1-1.788-2.662L7.07 10.48c-.462.462-.693.692-.891.947a5.2 5.2 0 0 0-.599.969c-.139.291-.242.601-.449 1.22l-1.088 3.267a.848.848 0 0 0 1.073 1.073l3.266-1.088c.62-.207.93-.31 1.221-.45q.518-.246.969-.598c.255-.199.485-.43.947-.891m7.56-7.559a3.146 3.146 0 0 0-4.45-4.449l-.71.71l.031.09c.26.749.751 1.732 1.674 2.655A7 7 0 0 0 18.37 8.08z"
      />
    </svg>
  );
}

export function TrashIcon({ size = 17, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.5"
        d="M9.17 4a3.001 3.001 0 0 1 5.66 0m5.67 2h-17m14.874 9.4c-.177 2.654-.266 3.981-1.131 4.79s-2.195.81-4.856.81h-.774c-2.66 0-3.99 0-4.856-.81c-.865-.809-.953-2.136-1.13-4.79l-.46-6.9m13.666 0l-.2 3M9.5 11l.5 5m4.5-5l-.5 5"
      />
    </svg>
  );
}

export function DiskIcon({ size = 24, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="currentColor"
        fillRule="evenodd"
        d="M20.536 20.536C22 19.07 22 16.714 22 12c0-.341 0-.512-.015-.686a4.04 4.04 0 0 0-.921-2.224a8 8 0 0 0-.483-.504l-5.167-5.167a9 9 0 0 0-.504-.483a4.04 4.04 0 0 0-2.224-.92C12.512 2 12.342 2 12 2C7.286 2 4.929 2 3.464 3.464C2 4.93 2 7.286 2 12s0 7.071 1.464 8.535c.685.685 1.563 1.05 2.786 1.243v-.83c0-.899 0-1.648.08-2.242c.084-.628.27-1.195.725-1.65c.456-.456 1.023-.642 1.65-.726c.595-.08 1.345-.08 2.243-.08h2.104c.899 0 1.648 0 2.242.08c.628.084 1.195.27 1.65.726c.456.455.642 1.022.726 1.65c.08.594.08 1.343.08 2.242v.83c1.223-.194 2.102-.558 2.785-1.242M6.25 8A.75.75 0 0 1 7 7.25h6a.75.75 0 0 1 0 1.5H7A.75.75 0 0 1 6.25 8"
        clipRule="evenodd"
      />
      <path
        fill="currentColor"
        d="M16.183 18.905c.065.483.067 1.131.067 2.095v.931C15.094 22 13.7 22 12 22s-3.094 0-4.25-.069V21c0-.964.002-1.612.067-2.095c.062-.461.169-.659.3-.789s.327-.237.788-.3c.483-.064 1.131-.066 2.095-.066h2c.964 0 1.612.002 2.095.067c.461.062.659.169.789.3s.237.327.3.788"
      />
    </svg>
  );
}

export function PlayIcon({ size = 24, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.5"
        d="M3 12v6.967c0 2.31 2.534 3.769 4.597 2.648l3.203-1.742M3 8V5.033c0-2.31 2.534-3.769 4.597-2.648l12.812 6.968a2.998 2.998 0 0 1 0 5.294l-6.406 3.484"
      />
    </svg>
  );
}

export function BookIcon({ size = 24, width, height, ...props }: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="currentColor"
        fillRule="evenodd"
        d="M8.945 1.25h6.11c1.367 0 2.47 0 3.337.117c.9.12 1.658.38 2.26.981c.602.602.86 1.36.982 2.26c.116.867.116 1.97.116 3.337v8.11c0 1.367 0 2.47-.116 3.337c-.122.9-.38 1.658-.982 2.26s-1.36.86-2.26.982c-.867.116-1.97.116-3.337.116h-6.11l-.899-.001a1 1 0 0 1-.1 0c-.918-.007-1.693-.029-2.338-.115c-.9-.122-1.658-.38-2.26-.982s-.86-1.36-.981-2.26c-.097-.715-.113-1.59-.116-2.642H2a.75.75 0 0 1 0-1.5h.25v-2.5H2a.75.75 0 0 1 0-1.5h.25v-2.5H2a.75.75 0 0 1 0-1.5h.25c.004-1.052.02-1.927.117-2.642c.12-.9.38-1.658.981-2.26c.602-.602 1.36-.86 2.26-.981c.867-.117 1.97-.117 3.337-.117M3.75 8.75H4a.75.75 0 0 0 0-1.5h-.25c.004-1.046.02-1.826.103-2.442c.099-.734.28-1.122.556-1.399c.277-.277.665-.457 1.4-.556c.4-.054.872-.08 1.441-.092V21.24a13 13 0 0 1-1.442-.092c-.734-.099-1.122-.28-1.399-.556c-.277-.277-.457-.665-.556-1.4c-.083-.615-.099-1.395-.102-2.441H4a.75.75 0 0 0 0-1.5h-.25v-2.5H4a.75.75 0 0 0 0-1.5h-.25zm5 12.5H15c1.435 0 2.436-.002 3.192-.103c.734-.099 1.122-.28 1.399-.556c.277-.277.457-.665.556-1.4c.101-.755.103-1.756.103-3.191V8c0-1.435-.002-2.437-.103-3.192c-.099-.734-.28-1.122-.556-1.399c-.277-.277-.665-.457-1.4-.556c-.755-.101-1.756-.103-3.191-.103H8.75zm2-14.75a.75.75 0 0 1 .75-.75h5a.75.75 0 0 1 0 1.5h-5a.75.75 0 0 1-.75-.75m0 3.5a.75.75 0 0 1 .75-.75h5a.75.75 0 0 1 0 1.5h-5a.75.75 0 0 1-.75-.75"
        clipRule="evenodd"
      />
    </svg>
  );
}

export const Spoiler: FC<{ children: ReactNode; className?: string }> = ({
  children,
  className,
}) => {
  const [hide, setHide] = useState(true);
  return (
    <div
      className={`relative rounded-xl overflow-hidden transition-all duration-300 ${hide ? "bg-slate-100/50 dark:bg-slate-800/30" : ""} ${className || ""}`}
    >
      <motion.div
        animate={{
          filter: hide ? "blur(8px)" : "blur(0px)",
          opacity: hide ? 0.3 : 1,
          scale: hide ? 0.98 : 1,
        }}
        transition={{ duration: 0.4, ease: "easeInOut" }}
        className={hide ? "select-none pointer-events-none grayscale" : ""}
        onClick={() => {
          if (!hide) return;
        }}
      >
        <div className="p-1">{children}</div>
      </motion.div>

      <AnimatePresence>
        {hide && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex items-center justify-center z-10 cursor-pointer group"
            onClick={() => setHide(false)}
          >
            <span className="bg-white/80 dark:bg-slate-900/80 backdrop-blur-md border border-slate-200 dark:border-slate-700 px-4 py-1.5 rounded-full text-xs font-bold uppercase tracking-widest text-slate-500 hover:text-blue-600 hover:border-blue-300 transition-all shadow-sm group-hover:scale-105 active:scale-95">
              Click to Reveal
            </span>
          </motion.div>
        )}
      </AnimatePresence>

      {!hide && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="flex justify-end p-2"
        >
          <button
            onClick={() => setHide(true)}
            className="text-[10px] text-slate-400 hover:text-blue-500 uppercase tracking-wider font-bold cursor-pointer transition-colors"
          >
            Hide again
          </button>
        </motion.div>
      )}
    </div>
  );
};

export function MoreVertIcon({
  size = 24,
  width,
  height,
  ...props
}: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size || width}
      height={size || height}
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        fill="currentColor"
        d="M12 16a2 2 0 0 1 2 2a2 2 0 0 1-2 2a2 2 0 0 1-2-2a2 2 0 0 1 2-2m0-6a2 2 0 0 1 2 2a2 2 0 0 1-2 2a2 2 0 0 1-2-2a2 2 0 0 1 2-2m0-6a2 2 0 0 1 2 2a2 2 0 0 1-2 2a2 2 0 0 1-2-2a2 2 0 0 1 2-2"
      />
    </svg>
  );
}

export function getPriorityColor(
  priority: number,
): "success" | "warning" | "secondary" {
  if (priority === 0) {
    return "success";
  } else if (priority === 1) {
    return "warning";
  } else {
    return "secondary";
  }
}

export function getPriorityText(priority: number) {
  if (priority === 0) {
    return "Low";
  } else if (priority === 1) {
    return "Medium";
  } else {
    return "High";
  }
}
export * from "./AudioPlayer";
export * from "./Markdown";
