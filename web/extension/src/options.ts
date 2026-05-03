import {
  DEFAULT_SETTINGS,
  type ExtensionSettings,
  type RuntimeRequest,
  type RuntimeResponse,
  type TranslationHistoryRecord,
} from "./shared";
import {
  BUILTIN_METHODS,
  HISTORY_LIMIT,
  LANGUAGES,
  PROMPT_MODES,
  SEARCH_ENGINES,
  type CustomLLMProvider,
  customMethodValue,
  methodLabel,
  type PromptMode,
} from "@/lib/translation";
import { getExtensionApi } from "./webextension";

const api = getExtensionApi();
const app = document.querySelector<HTMLElement>("#app");

if (!app) {
  throw new Error("Options root was not found.");
}

const appRoot: HTMLElement = app;

const HISTORY_RENDER_LIMIT = 30;
const MESSAGE_TIMEOUT_MS = 15000;

let settings: ExtensionSettings = DEFAULT_SETTINGS;
let history: TranslationHistoryRecord[] = [];
let statusText = "";
let statusKind: "idle" | "success" | "danger" = "idle";
let pendingAction = "";
let renderedShell = false;
let renderedHistorySignature = "";

function withTimeout<T>(promise: Promise<T>, timeoutMs: number, message: string) {
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

function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function option(value: string, label: string, selected: string) {
  return `<option value="${escapeHtml(value)}" ${selected === value ? "selected" : ""}>${escapeHtml(label)}</option>`;
}

function methodOptions() {
  const builtins = BUILTIN_METHODS.map((method) =>
    option(method.value, method.label, settings.method),
  ).join("");

  const custom = settings.customLLMs
    .flatMap((provider: CustomLLMProvider) =>
      provider.models.map((model) => {
        const value = customMethodValue({ name: provider.name, model });
        return option(value, `${model} (${provider.name})`, settings.method);
      }),
    )
    .join("");

  return `${builtins}${custom ? `<optgroup label="Custom LLM">${custom}</optgroup>` : ""}`;
}

function formatDate(value: number) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

function historyItems() {
  if (history.length === 0) {
    return `<p class="empty">No translation history yet.</p>`;
  }

  const visibleHistory = history.slice(0, HISTORY_RENDER_LIMIT);
  const hiddenCount = Math.max(history.length - visibleHistory.length, 0);
  const items = visibleHistory
    .map(
      (item) => `
        <article class="history-item">
          <div class="history-meta">
            <span>${escapeHtml(formatDate(item.createdAt))}</span>
            <span>${escapeHtml(methodLabel(item.method))}</span>
            <span>${item.wordType === 1 ? "Grammar" : "Word"}</span>
          </div>
          <p class="history-text">${escapeHtml(item.text)}</p>
          <p class="history-result">${escapeHtml(item.result)}</p>
          ${
            item.url
              ? `<a class="history-link" href="${escapeHtml(item.url)}" target="_blank" rel="noreferrer">${escapeHtml(item.title || item.url)}</a>`
              : ""
          }
          <div class="history-save-row">
            <button class="secondary history-save-toggle" type="button" data-history-id="${escapeHtml(item.id)}">Save</button>
            <div class="history-save-picker" data-history-picker="${escapeHtml(item.id)}" hidden>
              <span class="history-save-label">Save as</span>
              <button class="secondary history-save-option" type="button" data-history-id="${escapeHtml(item.id)}" data-word-type="0">Word</button>
              <button class="secondary history-save-option" type="button" data-history-id="${escapeHtml(item.id)}" data-word-type="1">Grammar</button>
              <button class="secondary history-save-cancel" type="button" data-history-id="${escapeHtml(item.id)}">Cancel</button>
            </div>
          </div>
        </article>
      `,
    )
    .join("");

  return `${items}${
    hiddenCount > 0
      ? `<p class="history-more">${hiddenCount} older records are kept but hidden here.</p>`
      : ""
  }`;
}

function historySignature() {
  const first = history[0]?.createdAt ?? 0;
  const last = history.length > 0 ? history[history.length - 1]?.createdAt ?? 0 : 0;
  return `${history.length}:${first}:${last}`;
}

function render() {
  if (renderedShell) {
    updateDynamicParts();
    return;
  }

  renderedShell = true;
  appRoot.innerHTML = `
    <style>
      :root {
        color-scheme: light dark;
        --dd-enji: #9f353a;
        --dd-yamabuki: #f8b500;
        --dd-gofun: #fffffb;
        --dd-sumi: #1c1c1c;
        --dd-nezumi: #787878;
        --dd-shironeri: #f3f3f2;
        --dd-bg: #f8f8f6;
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        background: var(--dd-bg);
        color: var(--dd-sumi);
      }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        min-width: 320px;
        background: var(--dd-bg);
        color: var(--dd-sumi);
      }
      .page {
        width: min(920px, calc(100vw - 32px));
        margin: 0 auto;
        padding: 32px 0 48px;
      }
      .header {
        display: flex;
        align-items: end;
        justify-content: space-between;
        gap: 20px;
        margin-bottom: 20px;
      }
      h1 {
        margin: 0;
        color: var(--dd-enji);
        font-size: 28px;
        line-height: 1.15;
        letter-spacing: 0;
      }
      .subtitle {
        margin: 6px 0 0;
        color: var(--dd-nezumi);
        font-size: 14px;
        line-height: 1.5;
      }
      .grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 16px;
      }
      .section {
        border: 1px solid #e6dedc;
        border-radius: 8px;
        background: var(--dd-gofun);
        padding: 18px;
      }
      .section.wide { grid-column: 1 / -1; }
      h2 {
        margin: 0 0 14px;
        font-size: 15px;
        line-height: 1.3;
        color: var(--dd-sumi);
        letter-spacing: 0;
      }
      label {
        display: grid;
        gap: 7px;
        margin-top: 12px;
        color: var(--dd-sumi);
        font-size: 13px;
        font-weight: 700;
      }
      input,
      select,
      textarea {
        width: 100%;
        min-height: 38px;
        border: 1px solid var(--dd-shironeri);
        border-radius: 7px;
        background: #ffffff;
        color: var(--dd-sumi);
        font: inherit;
        font-size: 14px;
        padding: 8px 10px;
        outline: none;
      }
      textarea {
        min-height: 96px;
        resize: vertical;
        line-height: 1.5;
      }
      input:focus,
      select:focus,
      textarea:focus {
        border-color: var(--dd-enji);
        box-shadow: 0 0 0 3px rgba(159, 53, 58, 0.14);
      }
      .row {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 12px;
      }
      .check-row {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-top: 14px;
        color: var(--dd-sumi);
        font-size: 13px;
        font-weight: 700;
      }
      .check-row input {
        width: 18px;
        height: 18px;
        min-height: 18px;
        accent-color: var(--dd-enji);
      }
      .actions {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        justify-content: flex-end;
        gap: 10px;
        margin-top: 18px;
      }
      button {
        min-height: 36px;
        border: 1px solid transparent;
        border-radius: 7px;
        padding: 0 14px;
        font: inherit;
        font-size: 13px;
        font-weight: 700;
        cursor: pointer;
      }
      button:disabled {
        cursor: wait;
        opacity: 0.62;
      }
      .primary {
        background: var(--dd-enji);
        color: white;
      }
      .secondary {
        border-color: var(--dd-shironeri);
        background: var(--dd-gofun);
        color: var(--dd-sumi);
      }
      .status {
        min-height: 22px;
        color: var(--dd-nezumi);
        font-size: 13px;
      }
      .status.success { color: #2d6a4f; }
      .status.danger { color: var(--dd-enji); }
      .status-banner {
        display: flex;
        align-items: center;
        min-height: 40px;
        margin: 0 0 16px;
        padding: 9px 12px;
        border: 1px solid var(--dd-shironeri);
        border-radius: 8px;
        background: var(--dd-gofun);
      }
      .status-banner.success {
        border-color: #b8d4c6;
        background: #f1f8f4;
      }
      .status-banner.danger {
        border-color: #e3c2c4;
        background: #fff4f4;
      }
      .hint {
        color: var(--dd-nezumi);
        font-size: 12px;
        line-height: 1.45;
        margin-top: 8px;
      }
      .history-actions {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        margin-bottom: 12px;
      }
      .history-actions .actions {
        margin-top: 0;
      }
      .history-count {
        color: var(--dd-nezumi);
        font-size: 12px;
      }
      .history-list {
        display: grid;
        gap: 10px;
        max-height: 520px;
        overflow: auto;
      }
      .history-item {
        border: 1px solid var(--dd-shironeri);
        border-radius: 7px;
        padding: 12px;
        background: #fafafa;
      }
      .history-meta {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        color: var(--dd-nezumi);
        font-size: 11px;
        line-height: 1.4;
      }
      .history-text,
      .history-result {
        margin: 8px 0 0;
        white-space: pre-wrap;
        word-break: break-word;
        line-height: 1.5;
      }
      .history-text {
        color: var(--dd-sumi);
        font-size: 13px;
        font-weight: 700;
      }
      .history-result {
        color: var(--dd-sumi);
        font-size: 13px;
      }
      .history-link {
        display: inline-block;
        margin-top: 8px;
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--dd-enji);
        font-size: 12px;
        text-decoration: none;
      }
      .history-save-row {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 8px;
        margin-top: 10px;
      }
      .history-save-picker {
        display: inline-flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 8px;
      }
      .history-save-label {
        color: var(--dd-nezumi);
        font-size: 11px;
        font-weight: 700;
      }
      .empty {
        margin: 0;
        color: var(--dd-nezumi);
        font-size: 13px;
      }
      .history-more {
        margin: 2px 0 0;
        color: var(--dd-nezumi);
        font-size: 12px;
        text-align: center;
      }
      @media (max-width: 720px) {
        .grid,
        .row {
          grid-template-columns: 1fr;
        }
        .header {
          display: block;
        }
      }
      @media (prefers-color-scheme: dark) {
        :root {
          --dd-gofun: #1c1c1c;
          --dd-sumi: #fffffb;
          --dd-nezumi: #b8b2aa;
          --dd-shironeri: #2b2927;
          --dd-bg: #141312;
        }
        input,
        select,
        textarea {
          background: #22201f;
        }
        .status-banner.success {
          border-color: #345c48;
          background: #1e2c25;
        }
        .status-banner.danger {
          border-color: #6b3134;
          background: #2c1d1e;
        }
        .history-item {
          background: #22201f;
        }
        .status.success { color: #95d5b2; }
      }
    </style>
    <div class="page">
      <header class="header">
        <div>
          <h1>DictDeck Extension</h1>
          <p class="subtitle">Selection translation and vocabulary capture.</p>
        </div>
        <button class="secondary load-custom" type="button">Load custom LLMs</button>
      </header>
      <p class="status status-banner ${statusKind}">${escapeHtml(
        statusText || "Settings changes will show a confirmation here.",
      )}</p>

      <div class="grid">
        <section class="section">
          <h2>Account</h2>
          <label>
            Service URL
            <input id="baseUrl" type="url" value="${escapeHtml(settings.baseUrl)}" placeholder="https://dictdeck.example.com" />
          </label>
          <label>
            Username
            <input id="username" autocomplete="username" value="${escapeHtml(settings.username)}" />
          </label>
          <label>
            Password
            <input id="password" type="password" autocomplete="current-password" placeholder="${settings.encryptedPassword ? "Saved. Fill only to replace." : "Saved encrypted after login"}" />
          </label>
          <div class="actions">
            <button class="secondary forget-password" type="button">Forget password</button>
            <button class="secondary save-settings" type="button" ${pendingAction ? "disabled" : ""}>${
              pendingAction === "save" ? "Saving..." : "Save settings"
            }</button>
            <button class="primary login" type="button" ${pendingAction ? "disabled" : ""}>${
              pendingAction === "login" ? "Logging in..." : "Log in"
            }</button>
          </div>
          <p class="hint account-hint">${
            settings.encryptedPassword
              ? "Encrypted password is saved for automatic re-login."
              : "No saved password. Automatic re-login is disabled."
          } ${settings.token ? "Token is saved." : "No token saved yet."}</p>
        </section>

        <section class="section">
          <h2>Query</h2>
          <label>
            Method
            <select id="method">${methodOptions()}</select>
          </label>
          <div class="row">
            <label>
              Source language
              <select id="srcLang">
                ${LANGUAGES.map((lang) => option(lang.value, lang.label, settings.srcLang)).join("")}
              </select>
            </label>
            <label>
              Target language
              <select id="dstLang">
                ${LANGUAGES.map((lang) => option(lang.value, lang.label, settings.dstLang)).join("")}
              </select>
            </label>
          </div>
          <label>
            Default save type
            <select id="defaultWordType">
              ${option("0", "Word", String(settings.defaultWordType))}
              ${option("1", "Grammar / Pattern", String(settings.defaultWordType))}
            </select>
          </label>
        </section>

        <section class="section wide">
          <h2>Custom LLM Options</h2>
          <div class="row">
            <label>
              Prompt mode
              <select id="promptMode">
                ${PROMPT_MODES.map((mode) => option(mode.value, mode.label, settings.promptMode)).join("")}
              </select>
            </label>
            <label>
              Search engine
              <select id="searchEngine">
                ${SEARCH_ENGINES.map((engine) => option(engine.value, engine.label, settings.searchEngine)).join("")}
              </select>
            </label>
          </div>
          <label class="check-row">
            <input id="googleSearch" type="checkbox" ${settings.googleSearch ? "checked" : ""} />
            Google Search
          </label>
          <label class="check-row">
            <input id="stream" type="checkbox" ${settings.stream ? "checked" : ""} />
            Stream custom LLM responses
          </label>
          <label>
            Instruction
            <textarea id="instruction" placeholder="Optional custom instruction">${escapeHtml(settings.instruction)}</textarea>
          </label>
          <div class="actions">
            <button class="primary save-settings" type="button" ${pendingAction ? "disabled" : ""}>${
              pendingAction === "save" ? "Saving..." : "Save settings"
            }</button>
          </div>
        </section>

        <section class="section wide">
          <div class="history-actions">
            <div>
              <h2>Translation History</h2>
              <span class="history-count">${history.length} / ${HISTORY_LIMIT} recent selections</span>
            </div>
            <div class="actions">
              <button class="secondary refresh-history" type="button">Refresh</button>
              <button class="secondary clear-history" type="button">Clear history</button>
            </div>
          </div>
          <div class="history-list">
            ${historyItems()}
          </div>
        </section>
      </div>
    </div>
  `;

  bindEvents();
  updateDynamicParts();
}

function updateDynamicParts() {
  const status = document.querySelector<HTMLElement>(".status-banner");
  if (status) {
    status.className = `status status-banner ${statusKind}`;
    status.textContent = statusText || "Settings changes will show a confirmation here.";
  }

  document.querySelectorAll<HTMLButtonElement>(".save-settings").forEach((button) => {
    button.disabled = Boolean(pendingAction);
    button.textContent = pendingAction === "save" ? "Saving..." : "Save settings";
  });

  const loginButton = document.querySelector<HTMLButtonElement>(".login");
  if (loginButton) {
    loginButton.disabled = Boolean(pendingAction);
    loginButton.textContent = pendingAction === "login" ? "Logging in..." : "Log in";
  }

  const password = document.querySelector<HTMLInputElement>("#password");
  if (password) {
    password.placeholder = settings.encryptedPassword
      ? "Saved. Fill only to replace."
      : "Saved encrypted after login";
  }

  const hint = document.querySelector<HTMLElement>(".account-hint");
  if (hint) {
    hint.textContent = `${
      settings.encryptedPassword
        ? "Encrypted password is saved for automatic re-login."
        : "No saved password. Automatic re-login is disabled."
    } ${settings.token ? "Token is saved." : "No token saved yet."}`;
  }

  const historyCount = document.querySelector<HTMLElement>(".history-count");
  if (historyCount) {
    historyCount.textContent = `${history.length} / ${HISTORY_LIMIT} recent selections`;
  }

  const historyList = document.querySelector<HTMLElement>(".history-list");
  const nextHistorySignature = historySignature();
  if (historyList && renderedHistorySignature !== nextHistorySignature) {
    renderedHistorySignature = nextHistorySignature;
    historyList.innerHTML = historyItems();
  }
}

function formSettings(): Partial<ExtensionSettings> {
  const value = (id: string) =>
    document.querySelector<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>(
      `#${id}`,
    )?.value ?? "";
  const checked = (id: string) =>
    document.querySelector<HTMLInputElement>(`#${id}`)?.checked ?? false;

  return {
    baseUrl: value("baseUrl"),
    username: value("username"),
    method: value("method"),
    srcLang: value("srcLang"),
    dstLang: value("dstLang"),
    instruction: value("instruction"),
    promptMode: value("promptMode") as PromptMode,
    googleSearch: checked("googleSearch"),
    searchEngine: value("searchEngine"),
    stream: checked("stream"),
    defaultWordType: value("defaultWordType") === "1" ? 1 : 0,
  };
}

function passwordValue() {
  return document.querySelector<HTMLInputElement>("#password")?.value ?? "";
}

function bindEvents() {
  document.querySelectorAll<HTMLButtonElement>(".save-settings").forEach((button) => {
    button.addEventListener("click", () => {
      void saveSettings();
    });
  });

  document.querySelector<HTMLButtonElement>(".login")?.addEventListener("click", () => {
    void login();
  });

  document.querySelector<HTMLButtonElement>(".load-custom")?.addEventListener("click", () => {
    void loadCustomLLMs();
  });

  document.querySelector<HTMLButtonElement>(".forget-password")?.addEventListener("click", () => {
    void forgetPassword();
  });

  document.querySelector<HTMLButtonElement>(".clear-history")?.addEventListener("click", () => {
    void clearHistory();
  });

  document.querySelector<HTMLButtonElement>(".refresh-history")?.addEventListener("click", () => {
    void refreshHistory();
  });

  document.querySelector<HTMLElement>(".history-list")?.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) return;

    const toggle = target.closest<HTMLButtonElement>(".history-save-toggle");
    if (toggle?.dataset.historyId) {
      toggleHistorySavePicker(toggle.dataset.historyId, true);
      return;
    }

    const cancel = target.closest<HTMLButtonElement>(".history-save-cancel");
    if (cancel?.dataset.historyId) {
      toggleHistorySavePicker(cancel.dataset.historyId, false);
      return;
    }

    const saveOption = target.closest<HTMLButtonElement>(".history-save-option");
    if (saveOption?.dataset.historyId) {
      const wordType = saveOption.dataset.wordType === "1" ? 1 : 0;
      toggleHistorySavePicker(saveOption.dataset.historyId, false);
      void saveHistoryItem(saveOption.dataset.historyId, wordType);
    }
  });
}

function toggleHistorySavePicker(historyId: string, visible: boolean) {
  document.querySelectorAll<HTMLElement>(".history-save-picker").forEach((picker) => {
    picker.hidden = picker.dataset.historyPicker !== historyId || !visible;
  });
}

function historySaveExample(item: TranslationHistoryRecord) {
  return [item.text, item.title, item.url].filter(Boolean).join("\n");
}

async function saveHistoryItem(historyId: string, wordType: 0 | 1) {
  const item = history.find((record) => record.id === historyId);
  if (!item) {
    setStatus("History item was not found.", "danger");
    return;
  }

  setPendingStatus("Saving history item...", "idle", "history-save");
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
    setStatus("History item saved.", "success");
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

async function saveSettings() {
  const nextSettings = formSettings();
  const password = passwordValue();
  setPendingStatus("Saving settings...", "idle", "save");
  try {
    settings = await sendMessage<ExtensionSettings>({
      type: "saveSettings",
      settings: nextSettings,
      password: password || undefined,
    });
    setStatus(
      password
        ? "Settings saved. Password encrypted for automatic login."
        : "Settings saved.",
      "success",
    );
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

async function login() {
  const nextSettings = formSettings();
  const password = passwordValue();
  setPendingStatus("Logging in...", "idle", "login");
  try {
    settings = await sendMessage<ExtensionSettings>({
      type: "saveSettings",
      settings: nextSettings,
      password: password || undefined,
    });
    settings = await sendMessage<ExtensionSettings>({
      type: "login",
      baseUrl: nextSettings.baseUrl || "",
      username: nextSettings.username || "",
      password,
    });
    setStatus("Logged in and settings saved.", "success");
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

async function forgetPassword() {
  try {
    settings = await sendMessage<ExtensionSettings>({ type: "forgetPassword" });
    setStatus("Saved password and token cleared.", "success");
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

async function clearHistory() {
  try {
    history = await sendMessage<TranslationHistoryRecord[]>({ type: "clearHistory" });
    setStatus("Translation history cleared.", "success");
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

async function refreshHistory() {
  try {
    history = await sendMessage<TranslationHistoryRecord[]>({ type: "getHistory" });
    setStatus("Translation history refreshed.", "success");
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

async function loadCustomLLMs() {
  try {
    settings = await sendMessage<ExtensionSettings>({
      type: "saveSettings",
      settings: formSettings(),
    });
    settings = await sendMessage<ExtensionSettings>({ type: "loadCustomLLMs" });
    setStatus("Custom LLM list loaded.", "success", true);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "danger");
  }
}

function setStatus(text: string, kind: "idle" | "success" | "danger", rebuild = false) {
  pendingAction = "";
  statusText = text;
  statusKind = kind;
  if (rebuild) {
    renderedShell = false;
  }
  render();
}

function setPendingStatus(
  text: string,
  kind: "idle" | "success" | "danger",
  action: string,
) {
  pendingAction = action;
  statusText = text;
  statusKind = kind;
  render();
}

async function init() {
  try {
    settings = await sendMessage<ExtensionSettings>({ type: "getSettings" });
    history = await sendMessage<TranslationHistoryRecord[]>({ type: "getHistory" });
  } catch (error) {
    statusText = error instanceof Error ? error.message : String(error);
    statusKind = "danger";
  }
  render();
}

void init();
