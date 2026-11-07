import { useEffect, useState, type ReactNode } from "react";
import {
  Bookmark,
  Check,
  CircleAlert,
  CircleDashed,
  Cpu,
  ExternalLink,
  History,
  KeyRound,
  Languages,
  Loader2,
  LogIn,
  RefreshCw,
  Save,
  Trash2,
  UserRound,
  X,
} from "lucide-react";
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
  formatRelativeTime,
  stripMarkdown,
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
const NAV_ITEMS = [
  { id: "account", label: "Account", Icon: UserRound },
  { id: "translation", label: "Translation", Icon: Languages },
  { id: "llm", label: "Custom LLM", Icon: Cpu },
  { id: "history", label: "History", Icon: History },
];
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

  useEffect(() => {
    if (statusKind !== "success" || !statusText) return;
    const timer = window.setTimeout(() => setStatusText(""), 3200);
    return () => window.clearTimeout(timer);
  }, [statusKind, statusText]);

  const visibleHistory = history.slice(0, HISTORY_RENDER_LIMIT);
  const hiddenCount = Math.max(history.length - visibleHistory.length, 0);
  const controlsDisabled = Boolean(pendingAction) || initializing;
  const toastVisible = Boolean(statusText) || initializing;
  const toastKind = initializing || pendingAction ? "pending" : statusKind;

  return (
    <div className="options-layout">
      <aside className="options-sidebar">
        <div className="options-brand">
          <img src="./assets/dictdeck.svg" alt="" aria-hidden="true" />
          <div>
            <strong>DictDeck</strong>
            <span>Extension settings</span>
          </div>
        </div>
        <nav className="options-nav" aria-label="Sections">
          {NAV_ITEMS.map(({ id, label, Icon }) => (
            <a key={id} href={`#${id}`}>
              <Icon aria-hidden="true" />
              {label}
            </a>
          ))}
        </nav>
        <div
          className={`options-connection ${settings.token ? "is-online" : ""}`}
        >
          <span className="options-connection-dot" />
          <span>
            {initializing
              ? "Loading…"
              : settings.token
                ? `Signed in as ${settings.username || "user"}`
                : "Not signed in"}
          </span>
        </div>
      </aside>

      <main className="options-main">
        <section className="options-section" id="account">
          <SectionHeading
            title="Account"
            description="Connect the extension to your DictDeck server. Passwords are encrypted locally for automatic re-login."
            aside={
              <span
                className={`dd-badge ${settings.token ? "" : "dd-badge--muted"}`}
              >
                {settings.token ? <Check /> : <CircleDashed />}
                {settings.token ? "Connected" : "Offline"}
              </span>
            }
          />
          <div className="options-card">
            <Field label="Service URL">
              <input
                className="dd-input"
                type="url"
                value={form.baseUrl}
                placeholder="https://dictdeck.example.com"
                onChange={(event) => updateForm("baseUrl", event.target.value)}
              />
            </Field>
            <div className="options-row">
              <Field label="Username">
                <input
                  className="dd-input"
                  autoComplete="username"
                  value={form.username}
                  onChange={(event) =>
                    updateForm("username", event.target.value)
                  }
                />
              </Field>
              <Field label="Password">
                <input
                  className="dd-input"
                  type="password"
                  autoComplete="current-password"
                  value={password}
                  placeholder={
                    settings.encryptedPassword
                      ? "Saved — fill only to replace"
                      : "Encrypted after login"
                  }
                  onChange={(event) => setPassword(event.target.value)}
                />
              </Field>
            </div>
            <div className="options-card-footer">
              <p className="options-hint">
                <KeyRound aria-hidden="true" />
                {settings.encryptedPassword
                  ? "Auto re-login enabled."
                  : "Auto re-login disabled."}
              </p>
              <div className="options-actions">
                <button
                  className="dd-button dd-button--quiet dd-button--danger"
                  type="button"
                  disabled={controlsDisabled}
                  onClick={() => void forgetPassword()}
                >
                  {pendingAction === "forget" ? "Clearing…" : "Forget password"}
                </button>
                <button
                  className="dd-button dd-button--primary"
                  type="button"
                  disabled={controlsDisabled}
                  onClick={() => void login()}
                >
                  <LogIn />
                  {pendingAction === "login" ? "Logging in…" : "Log in"}
                </button>
              </div>
            </div>
          </div>
        </section>

        <section className="options-section" id="translation">
          <SectionHeading
            title="Translation"
            description="Defaults for the selection popup. You can still switch these per lookup."
          />
          <div className="options-card">
            <Field label="Method">
              <select
                className="dd-select"
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
                    {settings.customLLMs.flatMap(
                      (provider: CustomLLMProvider) =>
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
            </Field>
            <div className="options-row">
              <Field label="Source language">
                <select
                  className="dd-select"
                  value={form.srcLang}
                  onChange={(event) =>
                    updateForm("srcLang", event.target.value)
                  }
                >
                  {LANGUAGES.map((language) => (
                    <option key={language.value} value={language.value}>
                      {language.label}
                    </option>
                  ))}
                </select>
              </Field>
              <Field label="Target language">
                <select
                  className="dd-select"
                  value={form.dstLang}
                  onChange={(event) =>
                    updateForm("dstLang", event.target.value)
                  }
                >
                  {LANGUAGES.map((language) => (
                    <option key={language.value} value={language.value}>
                      {language.label}
                    </option>
                  ))}
                </select>
              </Field>
            </div>
            <Field label="Default save type">
              <div className="options-segment" role="radiogroup">
                {(
                  [
                    [0, "Word"],
                    [1, "Grammar / Pattern"],
                  ] as const
                ).map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    role="radio"
                    aria-checked={form.defaultWordType === value}
                    className={form.defaultWordType === value ? "active" : ""}
                    onClick={() => updateForm("defaultWordType", value)}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </Field>
          </div>
        </section>

        <section className="options-section" id="llm">
          <SectionHeading
            title="Custom LLM"
            description="Options that apply when the method is a custom model from your server."
            aside={
              <button
                className="dd-button dd-button--sm"
                type="button"
                disabled={controlsDisabled}
                onClick={() => void loadCustomLLMs()}
              >
                <RefreshCw
                  className={
                    pendingAction === "load-custom" ? "options-spin" : ""
                  }
                />
                {pendingAction === "load-custom" ? "Loading…" : "Load models"}
              </button>
            }
          />
          <div className="options-card">
            <div className="options-row">
              <Field label="Prompt mode">
                <select
                  className="dd-select"
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
              </Field>
              <Field label="Search engine">
                <select
                  className="dd-select"
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
              </Field>
            </div>
            <div className="options-toggles">
              <Toggle
                label="Google Search"
                description="Let the model ground answers with web results."
                checked={form.googleSearch}
                onChange={(checked) => updateForm("googleSearch", checked)}
              />
              <Toggle
                label="Stream responses"
                description="Show the answer as it is being written."
                checked={form.stream}
                onChange={(checked) => updateForm("stream", checked)}
              />
            </div>
            <Field label="Instruction">
              <textarea
                className="dd-textarea options-textarea"
                value={form.instruction}
                placeholder="Optional extra instruction appended to every prompt"
                onChange={(event) =>
                  updateForm("instruction", event.target.value)
                }
              />
            </Field>
          </div>
        </section>

        <div className="options-savebar">
          <span>Changes apply to new lookups.</span>
          <button
            className="dd-button dd-button--primary"
            type="button"
            disabled={controlsDisabled}
            onClick={() => void saveSettings()}
          >
            <Save />
            {pendingAction === "save" ? "Saving…" : "Save settings"}
          </button>
        </div>

        <section className="options-section" id="history">
          <SectionHeading
            title="History"
            description={`${history.length} of ${HISTORY_LIMIT} recent selections are kept on this device.`}
            aside={
              <div className="options-actions">
                <button
                  className="dd-button dd-button--sm dd-button--icon"
                  type="button"
                  title="Refresh"
                  aria-label="Refresh history"
                  disabled={controlsDisabled}
                  onClick={() => void refreshHistory()}
                >
                  <RefreshCw
                    className={
                      pendingAction === "refresh-history" ? "options-spin" : ""
                    }
                  />
                </button>
                <button
                  className="dd-button dd-button--sm dd-button--danger"
                  type="button"
                  disabled={controlsDisabled || history.length === 0}
                  onClick={() => void clearHistory()}
                >
                  <Trash2 />
                  {pendingAction === "clear-history" ? "Clearing…" : "Clear"}
                </button>
              </div>
            }
          />

          {visibleHistory.length === 0 ? (
            <div className="options-empty">
              <History aria-hidden="true" />
              <p>No translation history yet.</p>
            </div>
          ) : (
            <ul className="options-card options-history-list">
              {visibleHistory.map((item, index) => (
                <li
                  className="options-history-item"
                  key={item.id}
                  style={{ animationDelay: `${Math.min(index, 12) * 30}ms` }}
                >
                  <div className="options-history-main">
                    <div className="options-history-head">
                      <span className="options-history-text">{item.text}</span>
                      <span
                        className={`dd-badge ${item.wordType === 1 ? "dd-badge--warning" : "dd-badge--muted"}`}
                      >
                        {item.wordType === 1 ? "Grammar" : "Word"}
                      </span>
                    </div>
                    <p className="options-history-result">
                      {stripMarkdown(item.result)}
                    </p>
                    <div className="options-history-meta">
                      <span>{formatRelativeTime(item.createdAt)}</span>
                      <span>{methodLabel(item.method)}</span>
                      {item.url && (
                        <a
                          className="options-history-link"
                          href={item.url}
                          target="_blank"
                          rel="noreferrer"
                        >
                          {item.title || item.url}
                          <ExternalLink aria-hidden="true" />
                        </a>
                      )}
                    </div>
                  </div>
                  <div className="options-history-save">
                    {historyPicker === item.id ? (
                      <div className="options-history-picker">
                        <button
                          className="dd-button dd-button--sm"
                          type="button"
                          disabled={Boolean(pendingAction)}
                          onClick={() => void saveHistoryItem(item.id, 0)}
                        >
                          Word
                        </button>
                        <button
                          className="dd-button dd-button--sm"
                          type="button"
                          disabled={Boolean(pendingAction)}
                          onClick={() => void saveHistoryItem(item.id, 1)}
                        >
                          Grammar
                        </button>
                        <button
                          className="dd-button dd-button--sm dd-button--quiet dd-button--icon"
                          type="button"
                          title="Cancel"
                          aria-label="Cancel"
                          disabled={Boolean(pendingAction)}
                          onClick={() => setHistoryPicker(undefined)}
                        >
                          <X />
                        </button>
                      </div>
                    ) : (
                      <button
                        className="dd-button dd-button--sm options-history-save-button"
                        type="button"
                        disabled={Boolean(pendingAction)}
                        onClick={() => setHistoryPicker(item.id)}
                      >
                        <Bookmark />
                        Save
                      </button>
                    )}
                  </div>
                </li>
              ))}
              {hiddenCount > 0 && (
                <li className="options-history-more">
                  {hiddenCount} older records are kept but hidden here.
                </li>
              )}
            </ul>
          )}
        </section>
      </main>

      <div
        className={`options-toast is-${toastKind} ${toastVisible ? "is-visible" : ""}`}
        role="status"
        aria-live="polite"
      >
        {toastKind === "pending" ? (
          <Loader2 className="options-spin" aria-hidden="true" />
        ) : toastKind === "danger" ? (
          <CircleAlert aria-hidden="true" />
        ) : (
          <Check aria-hidden="true" />
        )}
        <span>{initializing ? "Loading settings…" : statusText}</span>
        {toastKind === "danger" && (
          <button
            type="button"
            className="options-toast-close"
            aria-label="Dismiss"
            onClick={() => setStatusText("")}
          >
            <X />
          </button>
        )}
      </div>
    </div>
  );
}

function SectionHeading({
  title,
  description,
  aside,
}: {
  title: string;
  description: string;
  aside?: ReactNode;
}) {
  return (
    <header className="options-section-heading">
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {aside}
    </header>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="options-field">
      <span className="options-label">{label}</span>
      {children}
    </label>
  );
}

function Toggle({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="options-toggle">
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <input
        className="dd-switch"
        type="checkbox"
        role="switch"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}

createRoot(app).render(<OptionsApp />);
