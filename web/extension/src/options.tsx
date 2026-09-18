import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  BUILTIN_METHODS,
  HISTORY_LIMIT,
  LANGUAGES,
  PROMPT_MODES,
  SEARCH_ENGINES,
  customMethodValue,
  methodLabel,
  type PromptMode,
  type CustomLLMProvider,
} from "@/lib/translation";
import "./options.css";
import {
  DEFAULT_SETTINGS,
  type ExtensionSettings,
  type RuntimeRequest,
  type RuntimeResponse,
  type TranslationHistoryRecord,
} from "./shared";
import { getExtensionApi } from "./webextension";

const api = getExtensionApi();
const app = document.querySelector<HTMLElement>("#app");

if (!app) {
  throw new Error("Options root was not found.");
}

const HISTORY_RENDER_LIMIT = 30;
const MESSAGE_TIMEOUT_MS = 15000;

type StatusKind = "idle" | "success" | "danger";

type OptionsForm = {
  baseUrl: string;
  username: string;
  method: string;
  srcLang: string;
  dstLang: string;
  instruction: string;
  promptMode: PromptMode;
  googleSearch: boolean;
  searchEngine: string;
  stream: boolean;
  defaultWordType: 0 | 1;
};

function formFromSettings(settings: ExtensionSettings): OptionsForm {
  return {
    baseUrl: settings.baseUrl,
    username: settings.username,
    method: settings.method,
    srcLang: settings.srcLang,
    dstLang: settings.dstLang,
    instruction: settings.instruction,
    promptMode: settings.promptMode,
    googleSearch: settings.googleSearch,
    searchEngine: settings.searchEngine,
    stream: settings.stream,
    defaultWordType: settings.defaultWordType,
  };
}

function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message: string,
) {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<T>((_, reject) => {
    timeoutId = setTimeout(() => reject(new Error(message)), timeoutMs);
  });

  return Promise.race([promise, timeout]).finally(() => {
    if (timeoutId !== undefined) {
      clearTimeout(timeoutId);
    }
  });
}

async function sendMessage<T>(message: RuntimeRequest) {
  const response = (await withTimeout(
    api.runtime.sendMessage(message),
    MESSAGE_TIMEOUT_MS,
    "Extension background did not respond. Please reopen this page and try again.",
  )) as RuntimeResponse<T>;
  if (!response.ok) {
    throw new Error(response.error);
  }
  return response.data;
}

function historySaveExample(item: TranslationHistoryRecord) {
  return [item.text, item.title, item.url].filter(Boolean).join("\n");
}

function OptionsApp() {
  const [settings, setSettings] = useState<ExtensionSettings>(DEFAULT_SETTINGS);
  const [form, setForm] = useState<OptionsForm>(
    formFromSettings(DEFAULT_SETTINGS),
  );
  const [password, setPassword] = useState("");
  const [history, setHistory] = useState<TranslationHistoryRecord[]>([]);
  const [statusText, setStatusText] = useState("");
  const [statusKind, setStatusKind] = useState<StatusKind>("idle");
  const [pendingAction, setPendingAction] = useState("");
  const [historyPicker, setHistoryPicker] = useState<string>();
  const [initializing, setInitializing] = useState(true);

  useEffect(() => {
    let active = true;

    void Promise.all([
      sendMessage<ExtensionSettings>({ type: "getSettings" }),
      sendMessage<TranslationHistoryRecord[]>({ type: "getHistory" }),
    ])
      .then(([nextSettings, nextHistory]) => {
        if (!active) return;
        setSettings(nextSettings);
        setForm(formFromSettings(nextSettings));
        setHistory(nextHistory);
      })
      .catch((error: unknown) => {
        if (!active) return;
        setStatusText(error instanceof Error ? error.message : String(error));
        setStatusKind("danger");
      })
      .finally(() => {
        if (active) setInitializing(false);
      });

    return () => {
      active = false;
    };
  }, []);

  function updateForm<K extends keyof OptionsForm>(
    key: K,
    value: OptionsForm[K],
  ) {
    setForm((current) => ({ ...current, [key]: value }));
  }

  function setStatus(text: string, kind: StatusKind) {
    setPendingAction("");
    setStatusText(text);
    setStatusKind(kind);
  }

  function setPendingStatus(text: string, action: string) {
    setPendingAction(action);
    setStatusText(text);
    setStatusKind("idle");
  }

  async function saveSettings() {
    setPendingStatus("Saving settings...", "save");
    try {
      const nextSettings = await sendMessage<ExtensionSettings>({
        type: "saveSettings",
        settings: form,
        password: password || undefined,
      });
      setSettings(nextSettings);
      setStatus(
        password
          ? "Settings saved. Password encrypted for automatic login."
          : "Settings saved.",
        "success",
      );
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  async function login() {
    setPendingStatus("Logging in...", "login");
    try {
      const nextSettings = await sendMessage<ExtensionSettings>({
        type: "saveSettings",
        settings: form,
        password: password || undefined,
      });
      const loggedInSettings = await sendMessage<ExtensionSettings>({
        type: "login",
        baseUrl: form.baseUrl,
        username: form.username,
        password,
      });
      setSettings(loggedInSettings ?? nextSettings);
      setStatus("Logged in and settings saved.", "success");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  async function forgetPassword() {
    setPendingStatus("Clearing saved credentials...", "forget");
    try {
      const nextSettings = await sendMessage<ExtensionSettings>({
        type: "forgetPassword",
      });
      setSettings(nextSettings);
      setStatus("Saved password and token cleared.", "success");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  async function clearHistory() {
    setPendingStatus("Clearing translation history...", "clear-history");
    try {
      setHistory(
        await sendMessage<TranslationHistoryRecord[]>({
          type: "clearHistory",
        }),
      );
      setHistoryPicker(undefined);
      setStatus("Translation history cleared.", "success");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  async function refreshHistory() {
    setPendingStatus("Refreshing translation history...", "refresh-history");
    try {
      setHistory(
        await sendMessage<TranslationHistoryRecord[]>({
          type: "getHistory",
        }),
      );
      setStatus("Translation history refreshed.", "success");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  async function loadCustomLLMs() {
    setPendingStatus("Loading custom LLMs...", "load-custom");
    try {
      await sendMessage<ExtensionSettings>({
        type: "saveSettings",
        settings: form,
      });
      const nextSettings = await sendMessage<ExtensionSettings>({
        type: "loadCustomLLMs",
      });
      setSettings(nextSettings);
      setForm(formFromSettings(nextSettings));
      setStatus("Custom LLM list loaded.", "success");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  async function saveHistoryItem(historyId: string, wordType: 0 | 1) {
    const item = history.find((record) => record.id === historyId);
    if (!item) {
      setStatus("History item was not found.", "danger");
      return;
    }

    setPendingStatus("Saving history item...", "history-save");
    try {
      await sendMessage<object>({
        type: "saveWord",
        payload: {
          word: item.text,
          explain: item.result,
          example: historySaveExample(item),
          wordType,
        },
      });
      setHistoryPicker(undefined);
      setStatus("History item saved.", "success");
    } catch (error) {
      setStatus(
        error instanceof Error ? error.message : String(error),
        "danger",
      );
    }
  }

  const statusClass =
    statusKind === "success"
      ? "dd-status--success"
      : statusKind === "danger"
        ? "dd-status--danger"
        : "";
  const visibleHistory = history.slice(0, HISTORY_RENDER_LIMIT);
  const hiddenCount = Math.max(history.length - visibleHistory.length, 0);
  const controlsDisabled = Boolean(pendingAction) || initializing;

  return (
    <main className="options-page">
      <header className="options-header">
        <div>
          <p className="options-eyebrow">Workspace / Extension</p>
          <h1>Extension settings</h1>
          <p className="options-subtitle">
            Configure selection translation, saved credentials, and vocabulary
            capture from one shared DictDeck workspace.
          </p>
        </div>
        <button
          className="dd-button dd-button--primary"
          type="button"
          disabled={controlsDisabled}
          onClick={() => void loadCustomLLMs()}
        >
          {pendingAction === "load-custom" ? "Loading..." : "Load custom LLMs"}
        </button>
      </header>

      <p className={`dd-status options-status ${statusClass}`} role="status">
        {initializing
          ? "Loading extension settings..."
          : statusText || "Settings changes will show a confirmation here."}
      </p>

      <div className="options-grid">
        <section className="dd-panel options-section">
          <h2>Account</h2>
          <label className="options-field">
            Service URL
            <input
              className="dd-input"
              type="url"
              value={form.baseUrl}
              placeholder="https://dictdeck.example.com"
              onChange={(event) => updateForm("baseUrl", event.target.value)}
            />
          </label>
          <label className="options-field">
            Username
            <input
              className="dd-input"
              autoComplete="username"
              value={form.username}
              onChange={(event) => updateForm("username", event.target.value)}
            />
          </label>
          <label className="options-field">
            Password
            <input
              className="dd-input"
              type="password"
              autoComplete="current-password"
              value={password}
              placeholder={
                settings.encryptedPassword
                  ? "Saved. Fill only to replace."
                  : "Saved encrypted after login"
              }
              onChange={(event) => setPassword(event.target.value)}
            />
          </label>
          <div className="options-actions">
            <button
              className="dd-button"
              type="button"
              disabled={controlsDisabled}
              onClick={() => void forgetPassword()}
            >
              {pendingAction === "forget" ? "Clearing..." : "Forget password"}
            </button>
            <button
              className="dd-button"
              type="button"
              disabled={controlsDisabled}
              onClick={() => void saveSettings()}
            >
              {pendingAction === "save" ? "Saving..." : "Save settings"}
            </button>
            <button
              className="dd-button dd-button--primary"
              type="button"
              disabled={controlsDisabled}
              onClick={() => void login()}
            >
              {pendingAction === "login" ? "Logging in..." : "Log in"}
            </button>
          </div>
          <p className="options-hint">
            {settings.encryptedPassword
              ? "Encrypted password is saved for automatic re-login."
              : "No saved password. Automatic re-login is disabled."}{" "}
            {settings.token ? "Token is saved." : "No token saved yet."}
          </p>
        </section>

        <section className="dd-panel options-section">
          <h2>Query</h2>
          <label className="options-field">
            Method
            <select
              className="dd-select options-select"
              value={form.method}
              onChange={(event) => updateForm("method", event.target.value)}
            >
              {BUILTIN_METHODS.map((method) => (
                <option key={method.value} value={method.value}>
                  {method.label}
                </option>
              ))}
              {settings.customLLMs.length > 0 && (
                <optgroup label="Custom LLM">
                  {settings.customLLMs.flatMap((provider: CustomLLMProvider) =>
                    provider.models.map((model) => {
                      const value = customMethodValue({
                        name: provider.name,
                        model,
                      });
                      return (
                        <option key={value} value={value}>
                          {model} ({provider.name})
                        </option>
                      );
                    }),
                  )}
                </optgroup>
              )}
            </select>
          </label>
          <div className="options-row">
            <label className="options-field">
              Source language
              <select
                className="dd-select options-select"
                value={form.srcLang}
                onChange={(event) => updateForm("srcLang", event.target.value)}
              >
                {LANGUAGES.map((language) => (
                  <option key={language.value} value={language.value}>
                    {language.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="options-field">
              Target language
              <select
                className="dd-select options-select"
                value={form.dstLang}
                onChange={(event) => updateForm("dstLang", event.target.value)}
              >
                {LANGUAGES.map((language) => (
                  <option key={language.value} value={language.value}>
                    {language.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <label className="options-field">
            Default save type
            <select
              className="dd-select options-select"
              value={form.defaultWordType}
              onChange={(event) =>
                updateForm(
                  "defaultWordType",
                  event.target.value === "1" ? 1 : 0,
                )
              }
            >
              <option value="0">Word</option>
              <option value="1">Grammar / Pattern</option>
            </select>
          </label>
        </section>

        <section className="dd-panel options-section options-section--wide">
          <h2>Custom LLM options</h2>
          <div className="options-row">
            <label className="options-field">
              Prompt mode
              <select
                className="dd-select options-select"
                value={form.promptMode}
                onChange={(event) =>
                  updateForm("promptMode", event.target.value as PromptMode)
                }
              >
                {PROMPT_MODES.map((mode) => (
                  <option key={mode.value} value={mode.value}>
                    {mode.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="options-field">
              Search engine
              <select
                className="dd-select options-select"
                value={form.searchEngine}
                onChange={(event) =>
                  updateForm("searchEngine", event.target.value)
                }
              >
                {SEARCH_ENGINES.map((engine) => (
                  <option key={engine.value} value={engine.value}>
                    {engine.label}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <label className="options-checkbox">
            <input
              type="checkbox"
              checked={form.googleSearch}
              onChange={(event) =>
                updateForm("googleSearch", event.target.checked)
              }
            />
            Google Search
          </label>
          <label className="options-checkbox">
            <input
              type="checkbox"
              checked={form.stream}
              onChange={(event) => updateForm("stream", event.target.checked)}
            />
            Stream custom LLM responses
          </label>
          <label className="options-field">
            Instruction
            <textarea
              className="dd-textarea options-textarea"
              value={form.instruction}
              placeholder="Optional custom instruction"
              onChange={(event) =>
                updateForm("instruction", event.target.value)
              }
            />
          </label>
          <div className="options-actions">
            <button
              className="dd-button dd-button--primary"
              type="button"
              disabled={controlsDisabled}
              onClick={() => void saveSettings()}
            >
              {pendingAction === "save" ? "Saving..." : "Save settings"}
            </button>
          </div>
        </section>

        <section className="dd-panel options-section options-section--wide">
          <div className="options-history-heading">
            <div>
              <h2>Translation history</h2>
              <span className="options-history-count">
                {history.length} / {HISTORY_LIMIT} recent selections
              </span>
            </div>
            <div className="options-actions options-history-actions">
              <button
                className="dd-button"
                type="button"
                disabled={controlsDisabled}
                onClick={() => void refreshHistory()}
              >
                {pendingAction === "refresh-history"
                  ? "Refreshing..."
                  : "Refresh"}
              </button>
              <button
                className="dd-button"
                type="button"
                disabled={controlsDisabled || history.length === 0}
                onClick={() => void clearHistory()}
              >
                {pendingAction === "clear-history"
                  ? "Clearing..."
                  : "Clear history"}
              </button>
            </div>
          </div>

          <div className="options-history-list">
            {visibleHistory.length === 0 ? (
              <p className="dd-empty options-empty">
                No translation history yet.
              </p>
            ) : (
              visibleHistory.map((item) => (
                <article
                  className="dd-panel options-history-item"
                  key={item.id}
                >
                  <div className="options-history-meta">
                    <span>
                      {new Intl.DateTimeFormat(undefined, {
                        month: "short",
                        day: "2-digit",
                        hour: "2-digit",
                        minute: "2-digit",
                      }).format(new Date(item.createdAt))}
                    </span>
                    <span>{methodLabel(item.method)}</span>
                    <span>{item.wordType === 1 ? "Grammar" : "Word"}</span>
                  </div>
                  <p className="options-history-text">{item.text}</p>
                  <p className="options-history-result">{item.result}</p>
                  {item.url && (
                    <a
                      className="options-history-link"
                      href={item.url}
                      target="_blank"
                      rel="noreferrer"
                    >
                      {item.title || item.url}
                    </a>
                  )}
                  <div className="options-history-save-row">
                    <button
                      className="dd-button"
                      type="button"
                      disabled={Boolean(pendingAction)}
                      onClick={() =>
                        setHistoryPicker((current) =>
                          current === item.id ? undefined : item.id,
                        )
                      }
                    >
                      Save
                    </button>
                    {historyPicker === item.id && (
                      <div className="options-history-save-picker">
                        <span className="options-history-save-label">
                          Save as
                        </span>
                        <button
                          className="dd-button"
                          type="button"
                          disabled={Boolean(pendingAction)}
                          onClick={() => void saveHistoryItem(item.id, 0)}
                        >
                          Word
                        </button>
                        <button
                          className="dd-button"
                          type="button"
                          disabled={Boolean(pendingAction)}
                          onClick={() => void saveHistoryItem(item.id, 1)}
                        >
                          Grammar
                        </button>
                        <button
                          className="dd-button dd-button--quiet"
                          type="button"
                          disabled={Boolean(pendingAction)}
                          onClick={() => setHistoryPicker(undefined)}
                        >
                          Cancel
                        </button>
                      </div>
                    )}
                  </div>
                </article>
              ))
            )}
            {hiddenCount > 0 && (
              <p className="options-history-more">
                {hiddenCount} older records are kept but hidden here.
              </p>
            )}
          </div>
        </section>
      </div>
    </main>
  );
}

createRoot(app).render(<OptionsApp />);
