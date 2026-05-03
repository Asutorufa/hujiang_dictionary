import { methodLabel } from "@/lib/translation";
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

type PopupState = {
  settings?: ExtensionSettings;
  history: TranslationHistoryRecord[];
  error: string;
  loading: boolean;
};

const state: PopupState = {
  history: [],
  error: "",
  loading: true,
};

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(timestamp);
}

function escapeHtml(value: string) {
  return value
    .split("&").join("&amp;")
    .split("<").join("&lt;")
    .split(">").join("&gt;")
    .split('"').join("&quot;")
    .split("'").join("&#039;");
}

async function request<T>(message: unknown) {
  const response = (await api.runtime.sendMessage(message)) as RuntimeResponse<T>;
  if (!response.ok) {
    throw new Error(response.error);
  }
  return response.data;
}

async function openOptions() {
  await request<object>({ type: "openOptions" });
  window.close();
}

function renderHistory() {
  if (state.loading) {
    return `<div class="empty">Loading...</div>`;
  }

  if (state.history.length === 0) {
    return `<div class="empty">No selection history yet.</div>`;
  }

  return state.history
    .slice(0, 5)
    .map(
      (record) => `
        <article class="history-item">
          <div class="history-word">${escapeHtml(record.text)}</div>
          <div class="history-result">${escapeHtml(record.result || "No result")}</div>
          <div class="history-meta">${escapeHtml(methodLabel(record.method))} · ${formatTime(record.createdAt)}</div>
        </article>
      `,
    )
    .join("");
}

function render() {
  const configured = Boolean(state.settings?.baseUrl && state.settings?.username);
  const signedIn = Boolean(state.settings?.token);
  const service = state.settings?.baseUrl || "Not configured";
  const method = state.settings ? methodLabel(state.settings.method) : "Not configured";
  const target = state.settings?.dstLang || "Auto";

  root.innerHTML = `
    <style>
      :root {
        color-scheme: light dark;
        --enji: #9f353a;
        --yamabuki: #f8b500;
        --gofun: #fffffb;
        --sumi: #1c1c1c;
        --nezumi: #787878;
        --shirone: #f3f3f2;
        --panel: #fffffb;
        --panel-strong: #ffffff;
        --border: #deded8;
        --shadow: rgba(28, 28, 28, 0.12);
      }

      @media (prefers-color-scheme: dark) {
        :root {
          --gofun: #161615;
          --sumi: #f3f3f2;
          --nezumi: #a6a6a0;
          --shirone: #242421;
          --panel: #1c1c1c;
          --panel-strong: #242421;
          --border: #34342f;
          --shadow: rgba(0, 0, 0, 0.32);
        }
      }

      * {
        box-sizing: border-box;
      }

      body {
        width: 340px;
        min-height: 240px;
        margin: 0;
        color: var(--sumi);
        background: var(--gofun);
        font-family:
          Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      .shell {
        padding: 14px;
      }

      .header {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-bottom: 12px;
      }

      .mark {
        display: grid;
        width: 34px;
        height: 34px;
        place-items: center;
        border-radius: 8px;
        color: #fffffb;
        background: var(--enji);
        box-shadow: 0 8px 18px var(--shadow);
        font-size: 17px;
        font-weight: 760;
      }

      h1 {
        margin: 0;
        font-size: 15px;
        line-height: 1.2;
      }

      .sub {
        margin-top: 2px;
        color: var(--nezumi);
        font-size: 12px;
      }

      .status {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 9px 10px;
        border: 1px solid var(--border);
        border-radius: 8px;
        background: var(--panel);
      }

      .status-text {
        min-width: 0;
      }

      .status-title {
        overflow: hidden;
        font-size: 13px;
        font-weight: 700;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .status-detail {
        overflow: hidden;
        margin-top: 2px;
        color: var(--nezumi);
        font-size: 12px;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .pill {
        flex: 0 0 auto;
        padding: 4px 7px;
        border-radius: 999px;
        color: ${signedIn ? "var(--enji)" : "var(--nezumi)"};
        background: var(--shirone);
        font-size: 11px;
        font-weight: 700;
      }

      .grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
        margin: 10px 0;
      }

      .metric {
        min-width: 0;
        padding: 8px 9px;
        border: 1px solid var(--border);
        border-radius: 8px;
        background: var(--panel-strong);
      }

      .metric-label {
        color: var(--nezumi);
        font-size: 11px;
      }

      .metric-value {
        overflow: hidden;
        margin-top: 3px;
        font-size: 12px;
        font-weight: 650;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .actions {
        display: flex;
        gap: 8px;
        margin-bottom: 12px;
      }

      button {
        height: 32px;
        border: 1px solid var(--border);
        border-radius: 8px;
        color: var(--sumi);
        background: var(--panel-strong);
        cursor: pointer;
        font: inherit;
        font-size: 12px;
        font-weight: 700;
      }

      button.primary {
        flex: 1;
        border-color: var(--enji);
        color: #fffffb;
        background: var(--enji);
      }

      button.secondary {
        padding: 0 11px;
      }

      .section-title {
        margin: 0 0 7px;
        color: var(--nezumi);
        font-size: 11px;
        font-weight: 800;
        text-transform: uppercase;
      }

      .history {
        display: grid;
        max-height: 220px;
        gap: 7px;
        overflow: auto;
        padding-right: 2px;
      }

      .history-item {
        padding: 8px 9px;
        border: 1px solid var(--border);
        border-radius: 8px;
        background: var(--panel-strong);
      }

      .history-word {
        overflow: hidden;
        font-size: 13px;
        font-weight: 760;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .history-result {
        display: -webkit-box;
        margin-top: 4px;
        overflow: hidden;
        color: var(--sumi);
        font-size: 12px;
        line-height: 1.35;
        -webkit-box-orient: vertical;
        -webkit-line-clamp: 2;
      }

      .history-meta,
      .empty,
      .error {
        margin-top: 5px;
        color: var(--nezumi);
        font-size: 11px;
      }

      .empty,
      .error {
        margin-top: 0;
        padding: 14px;
        border: 1px dashed var(--border);
        border-radius: 8px;
        background: var(--panel);
        text-align: center;
      }

      .error {
        color: var(--enji);
      }

      .accent {
        color: var(--yamabuki);
      }
    </style>

    <main class="shell">
      <header class="header">
        <div class="mark">D</div>
        <div>
          <h1>DictDeck</h1>
          <div class="sub">Selection Translator</div>
        </div>
      </header>

      <section class="status">
        <div class="status-text">
          <div class="status-title">${escapeHtml(service)}</div>
          <div class="status-detail">${configured ? escapeHtml(state.settings?.username ?? "") : "Open settings to connect"}</div>
        </div>
        <div class="pill">${signedIn ? "Signed in" : "Setup"}</div>
      </section>

      <section class="grid">
        <div class="metric">
          <div class="metric-label">Method</div>
          <div class="metric-value">${escapeHtml(method)}</div>
        </div>
        <div class="metric">
          <div class="metric-label">Target</div>
          <div class="metric-value">${escapeHtml(target)}</div>
        </div>
      </section>

      <div class="actions">
        <button class="primary" type="button" data-action="options">${configured ? "Open Settings" : "Configure"}</button>
        <button class="secondary" type="button" data-action="reload">Refresh</button>
      </div>

      ${state.error ? `<div class="error">${escapeHtml(state.error)}</div>` : ""}

      <h2 class="section-title">Recent History</h2>
      <section class="history">${renderHistory()}</section>
    </main>
  `;

  root.querySelector<HTMLButtonElement>('[data-action="options"]')?.addEventListener("click", () => {
    openOptions().catch((error: unknown) => {
      state.error = error instanceof Error ? error.message : String(error);
      render();
    });
  });

  root.querySelector<HTMLButtonElement>('[data-action="reload"]')?.addEventListener("click", () => {
    load().catch((error: unknown) => {
      state.error = error instanceof Error ? error.message : String(error);
      state.loading = false;
      render();
    });
  });
}

async function load() {
  state.loading = true;
  state.error = "";
  render();

  const [settings, history] = await Promise.all([
    request<ExtensionSettings>({ type: "getSettings" }),
    request<TranslationHistoryRecord[]>({ type: "getHistory" }),
  ]);

  state.settings = settings;
  state.history = history;
  state.loading = false;
  render();
}

load().catch((error: unknown) => {
  state.error = error instanceof Error ? error.message : String(error);
  state.loading = false;
  render();
});
