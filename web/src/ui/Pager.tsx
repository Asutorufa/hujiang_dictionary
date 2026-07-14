import { IconButton, Text, TextField, Flex } from "@radix-ui/themes";
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";

export function Pager({
  page,
  total,
  onPageChange,
}: {
  page: number;
  total: number;
  onPageChange: (page: number) => void;
}) {
  const safeTotal = useMemo(() => Math.max(1, total || 1), [total]);
  const [draft, setDraft] = useState(String(page));

  useEffect(() => {
    setDraft(String(page));
  }, [page]);

  const commit = () => {
    const n = Number.parseInt(draft, 10);
    if (Number.isNaN(n)) {
      setDraft(String(page));
      return;
    }
    const clamped = Math.min(safeTotal, Math.max(1, n));
    setDraft(String(clamped));
    if (clamped !== page) onPageChange(clamped);
  };

  return (
    <Flex className="app-pager" gap="1" align="center">
      <IconButton
        size="1"
        variant="ghost"
        color="gray"
        disabled={page <= 1}
        onClick={() => onPageChange(1)}
        aria-label="First page"
      >
        <ChevronsLeft size={16} />
      </IconButton>
      <IconButton
        size="1"
        variant="ghost"
        color="gray"
        disabled={page <= 1}
        onClick={() => onPageChange(page - 1)}
        aria-label="Previous page"
      >
        <ChevronLeft size={16} />
      </IconButton>

      <TextField.Root
        size="1"
        variant="surface"
        className="w-[64px]"
        inputMode="numeric"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          if (e.key === "Escape") setDraft(String(page));
        }}
        aria-label="Page number"
      />
      <Text size="1" color="gray" className="tabular-nums px-1">
        / {safeTotal}
      </Text>

      <IconButton
        size="1"
        variant="ghost"
        color="gray"
        disabled={page >= safeTotal}
        onClick={() => onPageChange(page + 1)}
        aria-label="Next page"
      >
        <ChevronRight size={16} />
      </IconButton>
      <IconButton
        size="1"
        variant="ghost"
        color="gray"
        disabled={page >= safeTotal}
        onClick={() => onPageChange(safeTotal)}
        aria-label="Last page"
      >
        <ChevronsRight size={16} />
      </IconButton>
    </Flex>
  );
}
