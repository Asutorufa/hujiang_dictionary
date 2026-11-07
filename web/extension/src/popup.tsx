import { useCallback, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  ArrowUpRight,
  Languages,
  MousePointerClick,
  RotateCw,
  Settings,
  Sparkles,
} from "lucide-react";
import "./popup.css";
import { LANGUAGES, methodLabel } from "@/lib/translation";
import {
  formatRelativeTime,
  stripMarkdown,
  type ExtensionSettings,
  type RuntimeResponse,
  type TranslationHistoryRecord,
} from "./shared";
import { getExtensionApi } from "./webextension";

const api = getExtensionApi();
const root = document.querySelector<HTMLDivElement>("#root");

if (!root) {
  throw new Error("Popup root was not found.");
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

function hostOf(url: string) {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}

function HistoryList({
  history,
  loading,
}: {
  history: TranslationHistoryRecord[];
  loading: boolean;
}) {
  if (loading) {
    return (
      <ul className="popup-history" aria-busy="true">
        {[0, 1, 2].map((index) => (
          <li className="popup-history-item popup-skeleton" key={index}>
            <span className="skeleton-line skeleton-line--title" />
            <span className="skeleton-line" />
          </li>
        ))}
      </ul>
    );
  }

  if (history.length === 0) {
    return (
      <div className="popup-empty">
        <MousePointerClick aria-hidden="true" />
        <p>Select text on any page, then click the DictDeck bubble.</p>
      </div>
    );
  }

  return (
    <ul className="popup-history">
      {history.slice(0, 5).map((record, index) => (
        <li
          className="popup-history-item"
          key={record.id}
          style={{ animationDelay: `${index * 40}ms` }}
        >
          <div className="popup-history-head">
            <span className="popup-history-word">{record.text}</span>
            <time className="popup-history-time">
              {formatRelativeTime(record.createdAt)}
            </time>
          </div>
          <div className="popup-history-result">
            {stripMarkdown(record.result) || "No result"}
          </div>
        </li>
      ))}
    </ul>
  );
}

function PopupApp() {
  const [settings, setSettings] = useState<ExtensionSettings>();
  const [history, setHistory] = useState<TranslationHistoryRecord[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [spinKey, setSpinKey] = useState(0);

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
  const method = settings ? methodLabel(settings.method) : "—";
  const target =
    LANGUAGES.find((language) => language.value === settings?.dstLang)?.label ||
    "Auto";

  const handleOpenOptions = () => {
    void openOptions().catch((openError) => {
      setError(
        openError instanceof Error ? openError.message : String(openError),
      );
    });
  };

  return (
    <main className="popup-shell">
      <header className="popup-header">
        <img className="popup-mark" src="./assets/dictdeck.svg" alt="" />
        <div className="popup-title">
          <h1>DictDeck</h1>
          {settings && (
            <span
              className={`popup-state ${signedIn ? "is-online" : "is-setup"}`}
            >
              <span className="popup-state-dot" />
              {signedIn ? "Connected" : configured ? "Signed out" : "Setup"}
            </span>
          )}
        </div>
        <div className="popup-header-actions">
          <button
            className="dd-button dd-button--quiet dd-button--sm dd-button--icon"
            type="button"
            title="Refresh"
            aria-label="Refresh"
            onClick={() => {
              setSpinKey((key) => key + 1);
              void load();
            }}
          >
            <RotateCw key={spinKey} className="popup-spin" />
          </button>
          <button
            className="dd-button dd-button--quiet dd-button--sm dd-button--icon"
            type="button"
            title="Settings"
            aria-label="Settings"
            onClick={handleOpenOptions}
          >
            <Settings />
          </button>
        </div>
      </header>

      {settings && !configured ? (
        <section className="popup-setup">
          <Sparkles aria-hidden="true" />
          <div>
            <strong>Connect your DictDeck server</strong>
            <p>Add a service URL and sign in to start translating.</p>
          </div>
          <button
            className="dd-button dd-button--primary"
            type="button"
            onClick={handleOpenOptions}
          >
            Set up
            <ArrowUpRight />
          </button>
        </section>
      ) : (
        <section className="popup-summary" aria-label="Current settings">
          <div className="popup-summary-row">
            <span className="popup-summary-label">Server</span>
            <span className="popup-summary-value" title={settings?.baseUrl}>
              {settings ? hostOf(settings.baseUrl) : "…"}
            </span>
          </div>
          <div className="popup-summary-row">
            <span className="popup-summary-label">Method</span>
            <span className="popup-summary-value">{method}</span>
          </div>
          <div className="popup-summary-row">
            <span className="popup-summary-label">
              <Languages aria-hidden="true" />
              Target
            </span>
            <span className="popup-summary-value">{target}</span>
          </div>
        </section>
      )}

      {error && (
        <div className="dd-status dd-status--danger popup-error" role="alert">
          {error}
        </div>
      )}

      <div className="popup-section-head">
        <h2>Recent</h2>
        {history.length > 0 && (
          <span className="popup-section-count">{history.length}</span>
        )}
      </div>
      <HistoryList history={history} loading={loading} />
    </main>
  );
}

createRoot(root).render(<PopupApp />);
