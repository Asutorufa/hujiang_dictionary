import { Spinner } from "@radix-ui/themes";

export function LoadingOverlay({ show }: { show: boolean }) {
  if (!show) return null;
  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center app-overlay">
      <div className="rounded-2xl border border-[var(--gray-a6)] bg-[color-mix(in_srgb,var(--color-panel-solid)_88%,transparent)] px-5 py-4 shadow-md">
        <Spinner size="3" />
      </div>
    </div>
  );
}
