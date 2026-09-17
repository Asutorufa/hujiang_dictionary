import { IconButton, TextField } from "@radix-ui/themes";
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
    <nav className="app-pager" aria-label="Vocabulary pages">
      <div className="app-pager-group">
        <IconButton
          size="1"
          variant="ghost"
          color="gray"
          disabled={page <= 1}
          onClick={() => onPageChange(1)}
          aria-label="First page"
          title="First page"
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
          title="Previous page"
        >
          <ChevronLeft size={16} />
        </IconButton>
      </div>

      <div className="app-pager-jump">
        <span className="app-pager-jump-label">Jump to page</span>
        <div className="app-pager-current" aria-live="polite">
          <TextField.Root
            size="2"
            variant="surface"
            className="app-pager-input"
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
          <span>of</span>
          <strong>{safeTotal}</strong>
        </div>
        <button type="button" className="app-pager-go" onClick={commit}>
          Go
        </button>
      </div>

      <div className="app-pager-group">
        <IconButton
          size="1"
          variant="ghost"
          color="gray"
          disabled={page >= safeTotal}
          onClick={() => onPageChange(page + 1)}
          aria-label="Next page"
          title="Next page"
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
          title="Last page"
        >
          <ChevronsRight size={16} />
        </IconButton>
      </div>
    </nav>
  );
}
