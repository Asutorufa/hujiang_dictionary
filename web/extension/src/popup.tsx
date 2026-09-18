import { useCallback, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import "./popup.css";
import { LANGUAGES, methodLabel } from "@/lib/translation";
import type {
  ExtensionSettings,
  RuntimeResponse,
  TranslationHistoryRecord,
} from "./shared";
import { getExtensionApi } from "./webextension";

const api = getExtensionApi();
const root = document.querySelector<HTMLDivElement>("#root");

if (!root) {
  throw new Error("Popup root was not found.");
}

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(timestamp);
}

async function request<T>(message: unknown) {
  const response = (await api.runtime.sendMessage(
    message,
  )) as RuntimeResponse<T>;
  if (!response.ok) {
    throw new Error(response.error);
  }
  return response.data;
}

async function openOptions() {
  await request<object>({ type: "openOptions" });
  window.close();
}

function HistoryList({
  history,
  loading,
}: {
  history: TranslationHistoryRecord[];
  loading: boolean;
}) {
  if (loading) {
    return <div className="dd-empty popup-loading">Loading history...</div>;
  }

  if (history.length === 0) {
    return (
      <div className="dd-empty popup-loading">No selection history yet.</div>
    );
  }

  return (
    <>
      {history.slice(0, 5).map((record) => (
        <article className="dd-panel popup-history-item" key={record.id}>
          <div className="popup-history-word">{record.text}</div>
          <div className="popup-history-result">
            {record.result || "No result"}
          </div>
          <div className="popup-history-meta">
            {methodLabel(record.method)} · {formatTime(record.createdAt)}
          </div>
        </article>
      ))}
    </>
  );
}

function PopupApp() {
  const [settings, setSettings] = useState<ExtensionSettings>();
  const [history, setHistory] = useState<TranslationHistoryRecord[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");

    try {
      const [nextSettings, nextHistory] = await Promise.all([
        request<ExtensionSettings>({ type: "getSettings" }),
        request<TranslationHistoryRecord[]>({ type: "getHistory" }),
      ]);
      setSettings(nextSettings);
      setHistory(nextHistory);
    } catch (loadError) {
      setError(
        loadError instanceof Error ? loadError.message : String(loadError),
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const configured = Boolean(settings?.baseUrl && settings?.username);
  const signedIn = Boolean(settings?.token);
  const service = settings?.baseUrl || "Not configured";
  const method = settings ? methodLabel(settings.method) : "Not configured";
  const target =
    LANGUAGES.find((language) => language.value === settings?.dstLang)?.label ||
    "Auto";

  return (
    <main className="popup-shell">
      <header className="popup-brand">
        <img className="popup-mark" src="./assets/dictdeck.svg" alt="" />
        <div>
          <h1>DictDeck</h1>
          <div className="popup-subtitle">Selection translator</div>
        </div>
      </header>

      <section className="dd-panel popup-status">
        <div className="popup-status-copy">
          <div className="popup-status-title">{service}</div>
          <div className="popup-status-detail">
            {configured ? settings?.username : "Open settings to connect"}
          </div>
        </div>
        <span className="dd-badge popup-pill">
          {signedIn ? "Signed in" : "Setup"}
        </span>
      </section>

      <section className="popup-metrics" aria-label="Current settings">
        <div className="popup-metric">
          <div className="popup-metric-label">Method</div>
          <div className="popup-metric-value">{method}</div>
        </div>
        <div className="popup-metric">
          <div className="popup-metric-label">Target</div>
          <div className="popup-metric-value">{target}</div>
        </div>
      </section>

      <div className="popup-actions">
        <button
          className="dd-button dd-button--primary"
          type="button"
          onClick={() => {
            void openOptions().catch((openError) => {
              setError(
                openError instanceof Error
                  ? openError.message
                  : String(openError),
              );
            });
          }}
        >
          {configured ? "Open settings" : "Configure"}
        </button>
        <button
          className="dd-button"
          type="button"
          onClick={() => {
            void load();
          }}
        >
          Refresh
        </button>
      </div>

      {error && (
        <div className="dd-status dd-status--danger popup-error" role="alert">
          {error}
        </div>
      )}

      <h2 className="popup-section-title">Recent history</h2>
      <section className="popup-history">
        <HistoryList history={history} loading={loading} />
      </section>
    </main>
  );
}

createRoot(root).render(<PopupApp />);
