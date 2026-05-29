import {
  BUILTIN_METHODS,
  DEFAULT_METHOD,
  LANGUAGES,
  PROMPT_MODES,
  customMethodValue,
  type PromptMode,
} from "@/lib/translation";
import type {
  ExtensionSettings,
  QueryPortResponse,
  RuntimeRequest,
  RuntimeResponse,
  SaveWordPayload,
} from "./shared";
import { getExtensionApi, type ExtensionPort } from "./webextension";

const api = getExtensionApi();
const MAX_SELECTION_LENGTH = 2000;
const CONTEXT_LIMIT = 360;

type SelectionSnapshot = {
  text: string;
  example: string;
  rect: DOMRect;
  title: string;
  url: string;
};

let rootHost: HTMLDivElement | undefined;
let shadowRootRef: ShadowRoot | undefined;
let activePort: ExtensionPort | undefined;
let pendingSelection: SelectionSnapshot | undefined;
let currentSelection = "";
let currentResult = "";
let currentReasoning = "";
let currentExample = "";
let currentTitle = "";
let currentUrl = "";
let currentWordType: 0 | 1 = 0;
let currentMethod = DEFAULT_METHOD;
let currentDstLang = "";
let currentPromptMode: PromptMode = "default";
let selectionTimer: number | undefined;
let isPinned = false;
let manualPosition: { left: number; top: number } | undefined;
let reasoningCollapsed = false;
let reasoningAutoCollapsed = false;
let persistPreferencesTimer: number | undefined;

function styles() {
  return `
    :host {
      all: initial;
      color-scheme: light dark;
      --dd-enji: #9f353a;
      --dd-yamabuki: #f8b500;
      --dd-gofun: #fffffb;
      --dd-sumi: #1c1c1c;
      --dd-nezumi: #787878;
      --dd-shironeri: #f3f3f2;
      --dd-shadow: 0 18px 50px rgba(28, 28, 28, 0.18);
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    .panel {
      position: fixed;
      z-index: 2147483647;
      width: min(380px, calc(100vw - 24px));
      display: flex;
      flex-direction: column;
      overflow: hidden;
      border: 1px solid color-mix(in srgb, var(--dd-enji) 16%, var(--dd-shironeri));
      border-radius: 8px;
      background: var(--dd-gofun);
      box-shadow: var(--dd-shadow);
      color: var(--dd-sumi);
      letter-spacing: 0;
    }
    .trigger {
      position: fixed;
      z-index: 2147483647;
      width: 32px;
      height: 32px;
      display: inline-grid;
      place-items: center;
      border: 1px solid color-mix(in srgb, var(--dd-enji) 18%, var(--dd-shironeri));
      border-radius: 999px;
      background:
        radial-gradient(circle at top, rgba(248, 181, 0, 0.3), transparent 58%),
        color-mix(in srgb, var(--dd-gofun) 92%, var(--dd-enji) 8%);
      box-shadow: 0 12px 30px rgba(28, 28, 28, 0.18);
      color: var(--dd-enji);
      cursor: pointer;
      font: inherit;
      font-size: 12px;
      font-weight: 800;
      line-height: 1;
    }
    .trigger:hover {
      transform: translateY(-1px);
      box-shadow: 0 14px 34px rgba(28, 28, 28, 0.24);
    }
    .trigger:focus-visible {
      outline: 2px solid color-mix(in srgb, var(--dd-enji) 56%, transparent);
      outline-offset: 2px;
    }
    .topbar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 10px 12px;
      flex: 0 0 auto;
      border-bottom: 1px solid var(--dd-shironeri);
      background: color-mix(in srgb, var(--dd-enji) 6%, var(--dd-gofun));
      cursor: grab;
      user-select: none;
    }
    .topbar.dragging {
      cursor: grabbing;
    }
    .topbar-actions {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      flex: 0 0 auto;
    }
    .title {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: 13px;
      font-weight: 700;
      color: var(--dd-enji);
    }
    .icon-button {
      width: 28px;
      height: 28px;
      display: inline-grid;
      place-items: center;
      flex: 0 0 auto;
      border: 1px solid transparent;
      border-radius: 6px;
      background: transparent;
      color: var(--dd-nezumi);
      cursor: pointer;
      font: inherit;
    }
    .icon-button:hover {
      border-color: var(--dd-shironeri);
      background: rgba(28, 28, 28, 0.04);
      color: var(--dd-sumi);
    }
    .body {
      flex: 1 1 auto;
      min-height: 0;
      overflow: auto;
      overscroll-behavior: contain;
      padding: 12px;
    }
    .selected {
      margin: 0 0 10px;
      padding: 8px 10px;
      border-radius: 6px;
      background: var(--dd-shironeri);
      color: var(--dd-sumi);
      font-size: 13px;
      line-height: 1.5;
      white-space: pre-wrap;
      word-break: break-word;
    }
    .status {
      display: flex;
      align-items: center;
      gap: 8px;
      margin: 0 0 10px;
      color: var(--dd-nezumi);
      font-size: 12px;
      line-height: 1.4;
    }
    .quick-controls {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 8px;
      margin: 0 0 10px;
    }
    .quick-controls label {
      display: grid;
      gap: 4px;
      min-width: 0;
      color: var(--dd-nezumi);
      font-size: 11px;
      font-weight: 700;
      line-height: 1.2;
    }
    .quick-controls select {
      width: 100%;
      min-width: 0;
      min-height: 30px;
      border: 1px solid var(--dd-shironeri);
      border-radius: 7px;
      background: var(--dd-gofun);
      color: var(--dd-sumi);
      font: inherit;
      font-size: 12px;
      padding: 4px 8px;
      outline: none;
    }
    .quick-controls select:focus {
      border-color: var(--dd-enji);
      box-shadow: 0 0 0 3px color-mix(in srgb, var(--dd-enji) 14%, transparent);
    }
    .dot {
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: var(--dd-yamabuki);
      box-shadow: 0 0 0 4px color-mix(in srgb, var(--dd-yamabuki) 22%, transparent);
    }
    .result {
      margin: 0;
      color: var(--dd-sumi);
      font-size: 14px;
      line-height: 1.62;
      word-break: break-word;
    }
    .markdown {
      overflow-wrap: anywhere;
    }
    .markdown > :first-child {
      margin-top: 0;
    }
    .markdown > :last-child {
      margin-bottom: 0;
    }
    .markdown p,
    .markdown ul,
    .markdown ol,
    .markdown pre,
    .markdown blockquote,
    .markdown h1,
    .markdown h2,
    .markdown h3,
    .markdown h4 {
      margin: 0 0 10px;
    }
    .markdown ul,
    .markdown ol {
      padding-left: 18px;
    }
    .markdown ul {
      list-style: disc;
    }
    .markdown ol {
      list-style: decimal;
    }
    .markdown li + li {
      margin-top: 4px;
    }
    .markdown a {
      color: var(--dd-enji);
      text-decoration: underline;
      text-underline-offset: 2px;
      transition:
        color 140ms ease,
        background-color 140ms ease,
        text-decoration-color 140ms ease;
      cursor: pointer;
    }
    .markdown a:hover {
      color: color-mix(in srgb, var(--dd-enji) 84%, var(--dd-sumi));
      text-decoration-color: currentColor;
    }
    .markdown a:focus-visible {
      outline: 2px solid color-mix(in srgb, var(--dd-enji) 45%, transparent);
      outline-offset: 2px;
      border-radius: 4px;
      background: color-mix(in srgb, var(--dd-enji) 10%, transparent);
    }
    .markdown strong {
      font-weight: 700;
    }
    .markdown em {
      font-style: italic;
    }
    .markdown code {
      padding: 1px 6px;
      border-radius: 5px;
      border: 1px solid color-mix(in srgb, var(--dd-shironeri) 82%, transparent);
      background: color-mix(in srgb, var(--dd-shironeri) 86%, transparent);
      font-family: "SFMono-Regular", ui-monospace, "Cascadia Code", "JetBrains Mono", Consolas, monospace;
      font-size: 0.92em;
    }
    .markdown pre {
      padding: 10px;
      border-radius: 7px;
      border: 1px solid color-mix(in srgb, var(--dd-shironeri) 78%, transparent);
      background:
        linear-gradient(
          180deg,
          color-mix(in srgb, var(--dd-shironeri) 95%, transparent),
          color-mix(in srgb, var(--dd-shironeri) 88%, transparent)
        );
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.18);
      overflow: auto;
      tab-size: 2;
    }
    .markdown pre code {
      display: block;
      padding: 0;
      border: 0;
      background: transparent;
      white-space: pre;
      line-height: 1.6;
    }
    .markdown .code-block {
      margin: 0 0 10px;
      border: 1px solid color-mix(in srgb, var(--dd-shironeri) 78%, transparent);
      border-radius: 8px;
      overflow: hidden;
      background:
        linear-gradient(
          180deg,
          color-mix(in srgb, var(--dd-shironeri) 95%, transparent),
          color-mix(in srgb, var(--dd-shironeri) 88%, transparent)
        );
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.18);
    }
    .markdown .code-block-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
      min-height: 30px;
      padding: 0 10px;
      border-bottom: 1px solid color-mix(in srgb, var(--dd-shironeri) 78%, transparent);
      background: color-mix(in srgb, var(--dd-enji) 4%, var(--dd-gofun));
    }
    .markdown .code-language {
      color: var(--dd-nezumi);
      font-size: 11px;
      font-weight: 800;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }
    .markdown .code-block pre {
      margin: 0;
      border: 0;
      border-radius: 0;
      background: transparent;
      box-shadow: none;
    }
    .markdown .code-block pre code {
      padding: 0;
    }
    .markdown blockquote {
      padding: 8px 10px;
      border-left: 3px solid color-mix(in srgb, var(--dd-enji) 22%, var(--dd-shironeri));
      border-radius: 0 6px 6px 0;
      background: color-mix(in srgb, var(--dd-enji) 4%, var(--dd-gofun));
      color: var(--dd-nezumi);
    }
    .markdown h1,
    .markdown h2,
    .markdown h3,
    .markdown h4 {
      color: var(--dd-enji);
      line-height: 1.35;
    }
    .markdown h1 {
      font-size: 18px;
    }
    .markdown h2 {
      font-size: 16px;
    }
    .markdown h3 {
      font-size: 15px;
    }
    .markdown h4 {
      font-size: 14px;
    }
    .markdown hr {
      height: 1px;
      margin: 12px 0;
      border: 0;
      background: color-mix(in srgb, var(--dd-shironeri) 80%, var(--dd-enji) 20%);
    }
    .markdown .task-list {
      padding-left: 0;
      list-style: none;
    }
    .markdown .task-list-item {
      display: flex;
      align-items: flex-start;
      gap: 8px;
    }
    .markdown .task-check {
      margin: 3px 0 0;
      accent-color: var(--dd-enji);
      pointer-events: none;
    }
    .markdown-empty {
      color: var(--dd-nezumi);
    }
    .reasoning-wrap {
      margin: 0 0 10px;
      border: 1px solid var(--dd-shironeri);
      border-radius: 7px;
      background: rgba(28, 28, 28, 0.02);
      overflow: hidden;
    }
    .reasoning-toggle {
      width: 100%;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
      min-height: 34px;
      border: 0;
      background: transparent;
      color: var(--dd-nezumi);
      font: inherit;
      font-size: 12px;
      font-weight: 700;
      padding: 0 10px;
      cursor: pointer;
    }
    .reasoning-chevron {
      transition: transform 140ms ease;
    }
    .reasoning-wrap.collapsed .reasoning-chevron {
      transform: rotate(-90deg);
    }
    .reasoning {
      margin: 0;
      padding: 0 10px 10px;
      color: var(--dd-nezumi);
      font-size: 12px;
      line-height: 1.55;
      word-break: break-word;
    }
    .reasoning-wrap.collapsed .reasoning {
      display: none;
    }
    .footer {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
      padding: 10px 12px;
      flex: 0 0 auto;
      border-top: 1px solid var(--dd-shironeri);
      background: color-mix(in srgb, var(--dd-shironeri) 64%, var(--dd-gofun));
    }
    .footer-actions {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      flex: 0 0 auto;
    }
    .segment {
      display: inline-flex;
      overflow: hidden;
      border: 1px solid var(--dd-shironeri);
      border-radius: 7px;
      background: var(--dd-gofun);
    }
    .segment button,
    .primary,
    .link {
      min-height: 30px;
      border: 0;
      font: inherit;
      font-size: 12px;
      cursor: pointer;
    }
    .segment button {
      padding: 0 10px;
      background: transparent;
      color: var(--dd-nezumi);
    }
    .segment button.active {
      background: var(--dd-enji);
      color: white;
    }
    .primary {
      padding: 0 12px;
      border-radius: 7px;
      background: var(--dd-enji);
      color: white;
      font-weight: 700;
    }
    .primary:disabled {
      cursor: not-allowed;
      opacity: 0.52;
    }
    .secondary {
      min-height: 30px;
      padding: 0 12px;
      border: 1px solid var(--dd-shironeri);
      border-radius: 7px;
      background: var(--dd-gofun);
      color: var(--dd-sumi);
      font: inherit;
      font-size: 12px;
      font-weight: 700;
      cursor: pointer;
    }
    .secondary:disabled {
      cursor: not-allowed;
      opacity: 0.52;
    }
    .save-picker {
      display: none;
      align-items: center;
      gap: 8px;
      flex-wrap: wrap;
    }
    .save-picker.visible {
      display: inline-flex;
    }
    .save-picker-label {
      color: var(--dd-nezumi);
      font-size: 11px;
      font-weight: 700;
    }
    .pin-active {
      color: var(--dd-enji);
      border-color: var(--dd-shironeri);
      background: rgba(159, 53, 58, 0.08);
    }
    .link {
      padding: 0;
      background: transparent;
      color: var(--dd-enji);
      font-weight: 700;
    }
    @media (prefers-color-scheme: dark) {
      :host {
        --dd-gofun: #1c1c1c;
        --dd-sumi: #fffffb;
        --dd-nezumi: #b8b2aa;
        --dd-shironeri: #2b2927;
        --dd-shadow: 0 18px 50px rgba(0, 0, 0, 0.46);
      }
      .icon-button:hover {
        background: rgba(255, 255, 255, 0.06);
      }
      .trigger {
        background:
          radial-gradient(circle at top, rgba(248, 181, 0, 0.26), transparent 58%),
          color-mix(in srgb, var(--dd-gofun) 88%, var(--dd-enji) 12%);
        box-shadow: 0 12px 30px rgba(0, 0, 0, 0.38);
      }
      .trigger:hover {
        box-shadow: 0 14px 34px rgba(0, 0, 0, 0.44);
      }
      .markdown pre {
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
      }
      .markdown .code-block {
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
      }
    }
  `;
}

async function sendMessage<T>(message: RuntimeRequest) {
  const response = (await api.runtime.sendMessage(
    message,
  )) as RuntimeResponse<T>;
  if (!response.ok) {
    throw new Error(response.error);
  }
  return response.data;
}

function persistQuickPreferences() {
  if (persistPreferencesTimer !== undefined) {
    window.clearTimeout(persistPreferencesTimer);
  }

  persistPreferencesTimer = window.setTimeout(() => {
    void sendMessage<object>({
      type: "saveSettings",
      settings: {
        method: currentMethod,
        dstLang: currentDstLang,
        promptMode: currentPromptMode,
      },
    }).catch((error: unknown) => {
      console.warn("Failed to persist popup preferences.", error);
    });
  }, 120);
}

function ensureRoot() {
  if (shadowRootRef) return shadowRootRef;

  rootHost = document.createElement("div");
  rootHost.id = "dictdeck-extension-root";
  shadowRootRef = rootHost.attachShadow({ mode: "open" });
  const style = document.createElement("style");
  style.textContent = styles();
  shadowRootRef.append(style);
  document.documentElement.append(rootHost);
  return shadowRootRef;
}

function closePanel(resetPinned = true) {
  activePort?.disconnect();
  activePort = undefined;
  pendingSelection = undefined;
  currentSelection = "";
  currentResult = "";
  currentReasoning = "";
  currentExample = "";
  currentTitle = "";
  currentUrl = "";
  reasoningCollapsed = false;
  reasoningAutoCollapsed = false;
  manualPosition = undefined;
  if (resetPinned) {
    isPinned = false;
  }
  shadowRootRef?.querySelector(".panel")?.remove();
  shadowRootRef?.querySelector(".trigger")?.remove();
}

function isPanelEvent(event: Event) {
  if (!rootHost) return false;
  return event.composedPath().includes(rootHost);
}

function setStatus(text: string) {
  const status = shadowRootRef?.querySelector<HTMLElement>(".status-text");
  if (status) status.textContent = text;
}

function setPinned(nextPinned: boolean) {
  isPinned = nextPinned;
  const button = shadowRootRef?.querySelector<HTMLButtonElement>(".pin");
  if (!button) return;
  button.classList.toggle("pin-active", isPinned);
  button.setAttribute("aria-pressed", String(isPinned));
  button.title = isPinned ? "Unpin" : "Pin";
}

function setReasoningCollapsed(collapsed: boolean) {
  reasoningCollapsed = collapsed;
  const wrap = shadowRootRef?.querySelector<HTMLElement>(".reasoning-wrap");
  const button =
    shadowRootRef?.querySelector<HTMLButtonElement>(".reasoning-toggle");
  if (!wrap || !button) return;
  wrap.classList.toggle("collapsed", collapsed);
  button.setAttribute("aria-expanded", String(!collapsed));
}

function toggleSavePicker(force?: boolean) {
  const picker = shadowRootRef?.querySelector<HTMLElement>(".save-picker");
  if (!picker) return;
  const visible = force ?? !picker.classList.contains("visible");
  picker.classList.toggle("visible", visible);
}

function setResult(result: string, reasoning?: string) {
  const previousResult = currentResult;
  currentResult = result;
  currentReasoning = reasoning || "";
  const resultEl = shadowRootRef?.querySelector<HTMLElement>(".result");
  const reasoningWrap =
    shadowRootRef?.querySelector<HTMLElement>(".reasoning-wrap");
  const reasoningEl = shadowRootRef?.querySelector<HTMLElement>(".reasoning");
  const saveButton = shadowRootRef?.querySelector<HTMLButtonElement>(".save");

  if (resultEl) {
    resultEl.innerHTML = currentResult
      ? renderMarkdown(currentResult)
      : '<p class="markdown-empty">No result yet.</p>';
  }
  if (reasoningWrap) {
    reasoningWrap.hidden = !currentReasoning;
  }
  if (reasoningEl) reasoningEl.innerHTML = renderMarkdown(currentReasoning);
  if (currentReasoning && !currentResult && !reasoningAutoCollapsed) {
    setReasoningCollapsed(false);
  }
  if (
    !previousResult &&
    currentResult &&
    currentReasoning &&
    !reasoningAutoCollapsed
  ) {
    reasoningAutoCollapsed = true;
    setReasoningCollapsed(true);
  }
  if (saveButton) saveButton.disabled = !currentResult;
}

function setTranslateBusy(isBusy: boolean) {
  const button = shadowRootRef?.querySelector<HTMLButtonElement>(".translate");
  if (!button) return;
  button.disabled = isBusy;
  button.textContent = isBusy ? "Translating" : "Translate";
}

function setWordType(wordType: 0 | 1) {
  currentWordType = wordType;
  shadowRootRef
    ?.querySelectorAll<HTMLButtonElement>("[data-word-type]")
    .forEach((button) => {
      const active = button.dataset.wordType === String(wordType);
      button.classList.toggle("active", active);
      button.setAttribute("aria-pressed", String(active));
    });
}

function option(value: string, label: string, selected: string) {
  return `<option value="${escapeAttribute(value)}" ${selected === value ? "selected" : ""}>${escapeAttribute(label)}</option>`;
}

function languageOptions(selected: string) {
  return LANGUAGES.map((language) =>
    option(language.value, language.label, selected),
  ).join("");
}

function promptModeOptions(selected: PromptMode) {
  return PROMPT_MODES.map((mode) =>
    option(mode.value, mode.label, selected),
  ).join("");
}

function methodOptions(settings: ExtensionSettings) {
  const builtins = BUILTIN_METHODS.map((method) =>
    option(method.value, method.label, currentMethod || settings.method),
  ).join("");

  const custom = settings.customLLMs
    .flatMap((provider) =>
      provider.models.map((model) =>
        option(
          customMethodValue({ name: provider.name, model }),
          `${model} (${provider.name})`,
          currentMethod || settings.method,
        ),
      ),
    )
    .join("");

  return `${builtins}${custom ? `<optgroup label="Custom LLM">${custom}</optgroup>` : ""}`;
}

function syncQuickControls() {
  currentMethod =
    shadowRootRef?.querySelector<HTMLSelectElement>(".method")?.value ??
    currentMethod;
  currentDstLang =
    shadowRootRef?.querySelector<HTMLSelectElement>(".dst-lang")?.value ??
    currentDstLang;
  currentPromptMode = (shadowRootRef?.querySelector<HTMLSelectElement>(
    ".prompt-mode",
  )?.value ?? currentPromptMode) as PromptMode;
  persistQuickPreferences();
}

function createSelectionSnapshot(selection: Selection) {
  const text = selection.toString().trim();
  const rect = selectionRect(selection);
  if (!text || !rect) return undefined;

  return {
    text,
    example: selectedContext(selection),
    rect: new DOMRect(rect.x, rect.y, rect.width, rect.height),
    title: document.title,
    url: location.href,
  } satisfies SelectionSnapshot;
}

function selectedContext(selection: Selection) {
  const range = selection.rangeCount > 0 ? selection.getRangeAt(0) : undefined;
  const container = range?.commonAncestorContainer;
  const element =
    container instanceof Element ? container : container?.parentElement;
  const sourceText = element?.textContent?.replace(/\s+/g, " ").trim() || "";
  const selected = selection.toString().replace(/\s+/g, " ").trim();
  const index = sourceText.indexOf(selected);

  if (index === -1) {
    return `${selected}\n\n${document.title}\n${location.href}`;
  }

  const start = Math.max(0, index - CONTEXT_LIMIT / 2);
  const end = Math.min(
    sourceText.length,
    index + selected.length + CONTEXT_LIMIT / 2,
  );
  const prefix = start > 0 ? "..." : "";
  const suffix = end < sourceText.length ? "..." : "";
  return `${prefix}${sourceText.slice(start, end)}${suffix}\n\n${document.title}\n${location.href}`;
}

function selectionRect(selection: Selection) {
  if (selection.rangeCount === 0) return undefined;
  const range = selection.getRangeAt(0);
  const rects = Array.from(range.getClientRects());
  return rects[0] || range.getBoundingClientRect();
}

function placeTrigger(trigger: HTMLElement, rect: DOMRect) {
  const margin = 8;
  const offset = 10;
  const left = Math.max(
    margin,
    Math.min(
      rect.left + rect.width / 2 - trigger.offsetWidth / 2,
      window.innerWidth - trigger.offsetWidth - margin,
    ),
  );
  const availableBelow = window.innerHeight - rect.bottom - margin;
  const availableAbove = rect.top - margin;
  const shouldPlaceBelow =
    availableBelow >= trigger.offsetHeight + offset ||
    availableBelow >= availableAbove;
  const top = shouldPlaceBelow
    ? Math.min(
        rect.bottom + offset,
        window.innerHeight - trigger.offsetHeight - margin,
      )
    : Math.max(margin, rect.top - trigger.offsetHeight - offset);

  trigger.style.left = `${left}px`;
  trigger.style.top = `${top}px`;
}

function placePanel(panel: HTMLElement, rect: DOMRect) {
  const margin = 12;
  const desiredHeight = Math.min(520, window.innerHeight - margin * 2);
  const availableBelow = window.innerHeight - rect.bottom - margin * 2;
  const availableAbove = rect.top - margin * 2;
  const shouldPlaceBelow =
    availableBelow >= Math.min(260, desiredHeight) ||
    availableBelow >= availableAbove;

  const preferredLeft = rect.left + rect.width / 2 - panel.offsetWidth / 2;
  const left = Math.max(
    margin,
    Math.min(preferredLeft, window.innerWidth - panel.offsetWidth - margin),
  );
  const top = shouldPlaceBelow
    ? rect.bottom + margin
    : Math.max(margin, rect.top - desiredHeight - margin);
  const maxHeight = shouldPlaceBelow
    ? Math.max(160, window.innerHeight - top - margin)
    : Math.max(160, rect.top - margin * 2);

  const finalLeft = manualPosition
    ? Math.max(
        margin,
        Math.min(
          manualPosition.left,
          window.innerWidth - panel.offsetWidth - margin,
        ),
      )
    : left;
  const finalTop = manualPosition
    ? Math.max(
        margin,
        Math.min(manualPosition.top, window.innerHeight - 160 - margin),
      )
    : Math.max(margin, top);

  panel.style.left = `${finalLeft}px`;
  panel.style.top = `${finalTop}px`;
  panel.style.maxHeight = `${Math.min(desiredHeight, maxHeight)}px`;
}

function trapFloatingElementEvents(element: HTMLElement) {
  for (const eventName of [
    "pointerdown",
    "pointerup",
    "mousedown",
    "mouseup",
    "click",
  ]) {
    element.addEventListener(eventName, (event) => event.stopPropagation());
  }
}

function enableDragging(panel: HTMLElement) {
  const topbar = panel.querySelector<HTMLElement>(".topbar");
  if (!topbar) return;

  let stopDragging: (() => void) | undefined;

  topbar.addEventListener("pointerdown", (event) => {
    const target = event.target;
    if (
      !(target instanceof Element) ||
      target.closest("button, select, input, textarea")
    ) {
      return;
    }

    const panelRect = panel.getBoundingClientRect();
    const offsetX = event.clientX - panelRect.left;
    const offsetY = event.clientY - panelRect.top;
    let hasMoved = false;
    stopDragging?.();
    topbar.setPointerCapture?.(event.pointerId);
    event.preventDefault();

    const move = (moveEvent: PointerEvent) => {
      if (
        !hasMoved &&
        Math.abs(moveEvent.clientX - event.clientX) +
          Math.abs(moveEvent.clientY - event.clientY) <
          6
      ) {
        return;
      }
      if (!hasMoved) {
        hasMoved = true;
        topbar.classList.add("dragging");
      }
      const margin = 12;
      const left = Math.max(
        margin,
        Math.min(
          moveEvent.clientX - offsetX,
          window.innerWidth - panel.offsetWidth - margin,
        ),
      );
      const top = Math.max(
        margin,
        Math.min(
          moveEvent.clientY - offsetY,
          window.innerHeight - panel.offsetHeight - margin,
        ),
      );
      manualPosition = { left, top };
      panel.style.left = `${left}px`;
      panel.style.top = `${top}px`;
    };

    const stop = () => {
      topbar.classList.remove("dragging");
      if (topbar.hasPointerCapture?.(event.pointerId)) {
        topbar.releasePointerCapture(event.pointerId);
      }
      document.removeEventListener("pointermove", move, true);
      topbar.removeEventListener("pointerup", stop);
      topbar.removeEventListener("pointercancel", stop);
      topbar.removeEventListener("lostpointercapture", stop);
      stopDragging = undefined;
    };

    stopDragging = stop;
    document.addEventListener("pointermove", move, true);
    topbar.addEventListener("pointerup", stop, { once: true });
    topbar.addEventListener("pointercancel", stop, { once: true });
    topbar.addEventListener("lostpointercapture", stop, { once: true });
  });
}

function renderTrigger(snapshot: SelectionSnapshot) {
  const root = ensureRoot();
  closePanel();
  pendingSelection = snapshot;

  const trigger = document.createElement("button");
  trigger.className = "trigger";
  trigger.type = "button";
  trigger.title = "Open DictDeck";
  trigger.setAttribute("aria-label", "Open DictDeck");
  trigger.textContent = "D";
  trigger.addEventListener("click", () => {
    void openPendingSelection();
  });
  trapFloatingElementEvents(trigger);
  root.append(trigger);
  placeTrigger(trigger, snapshot.rect);
}

function renderPanel(snapshot: SelectionSnapshot, settings: ExtensionSettings) {
  const root = ensureRoot();
  closePanel();

  currentSelection = snapshot.text;
  currentExample = snapshot.example;
  currentTitle = snapshot.title;
  currentUrl = snapshot.url;
  currentWordType = settings.defaultWordType;
  currentMethod = settings.method;
  currentDstLang = settings.dstLang;
  currentPromptMode = settings.promptMode;
  currentResult = "";
  currentReasoning = "";
  reasoningCollapsed = false;
  reasoningAutoCollapsed = false;
  manualPosition = undefined;

  const panel = document.createElement("section");
  panel.className = "panel";
  panel.innerHTML = `
    <div class="topbar">
      <div class="title" title="${escapeAttribute(currentSelection)}">DictDeck</div>
      <div class="topbar-actions">
        <button class="icon-button pin" type="button" title="Pin" aria-label="Pin" aria-pressed="false">📌</button>
        <button class="icon-button close" type="button" title="Close" aria-label="Close">×</button>
      </div>
    </div>
    <div class="body">
      <p class="selected"></p>
      <div class="quick-controls">
        <label>
          Method
          <select class="method">${methodOptions(settings)}</select>
        </label>
        <label>
          Target
          <select class="dst-lang">${languageOptions(currentDstLang)}</select>
        </label>
        <label>
          Prompt
          <select class="prompt-mode">${promptModeOptions(currentPromptMode)}</select>
        </label>
      </div>
      <p class="status"><span class="dot"></span><span class="status-text">Ready.</span></p>
      <div class="reasoning-wrap collapsed" hidden>
        <button class="reasoning-toggle" type="button" aria-expanded="false">
          <span>Think</span>
          <span class="reasoning-chevron">▾</span>
        </button>
        <div class="reasoning markdown"></div>
      </div>
      <div class="result markdown"><p class="markdown-empty">No result yet.</p></div>
    </div>
    <div class="footer">
      <div class="save-picker" aria-label="Save type">
        <span class="save-picker-label">Save as</span>
        <div class="segment">
          <button type="button" data-word-type="0">Word</button>
          <button type="button" data-word-type="1">Grammar</button>
        </div>
        <button class="secondary cancel-save" type="button">Cancel</button>
      </div>
      <div class="footer-actions">
        <button class="secondary translate" type="button">Translate</button>
        <button class="primary save" type="button" disabled>Save</button>
      </div>
    </div>
  `;

  root.append(panel);
  panel.querySelector<HTMLElement>(".selected")!.textContent = currentSelection;
  panel
    .querySelector<HTMLButtonElement>(".pin")!
    .addEventListener("click", () => {
      setPinned(!isPinned);
    });
  panel
    .querySelector<HTMLButtonElement>(".close")!
    .addEventListener("click", () => {
      closePanel();
    });
  panel
    .querySelector<HTMLSelectElement>(".method")!
    .addEventListener("change", () => {
      syncQuickControls();
    });
  panel
    .querySelector<HTMLSelectElement>(".dst-lang")!
    .addEventListener("change", () => {
      syncQuickControls();
    });
  panel
    .querySelector<HTMLSelectElement>(".prompt-mode")!
    .addEventListener("change", () => {
      syncQuickControls();
    });
  panel
    .querySelector<HTMLButtonElement>(".reasoning-toggle")!
    .addEventListener("click", () => {
      setReasoningCollapsed(!reasoningCollapsed);
    });
  panel
    .querySelectorAll<HTMLButtonElement>("[data-word-type]")
    .forEach((button) => {
      button.addEventListener("click", () => {
        setWordType(button.dataset.wordType === "1" ? 1 : 0);
        toggleSavePicker(false);
        void saveCurrentWord();
      });
    });
  panel
    .querySelector<HTMLButtonElement>(".cancel-save")!
    .addEventListener("click", () => {
      toggleSavePicker(false);
    });
  panel
    .querySelector<HTMLButtonElement>(".save")!
    .addEventListener("click", () => {
      toggleSavePicker();
    });
  panel
    .querySelector<HTMLButtonElement>(".translate")!
    .addEventListener("click", () => {
      startTranslation();
    });
  trapFloatingElementEvents(panel);
  setWordType(currentWordType);
  setReasoningCollapsed(true);
  setPinned(false);
  enableDragging(panel);
  placePanel(panel, snapshot.rect);

  setResult("");
}

function escapeAttribute(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function sanitizeHref(rawHref: string) {
  const href = rawHref.trim();
  if (!href) return "";

  if (
    href.startsWith("/") ||
    href.startsWith("./") ||
    href.startsWith("../") ||
    href.startsWith("#")
  ) {
    return href;
  }

  try {
    const url = new URL(href);
    if (
      url.protocol === "http:" ||
      url.protocol === "https:" ||
      url.protocol === "mailto:"
    ) {
      return url.toString();
    }
  } catch {
    return "";
  }

  return "";
}

function createLinkToken(tokens: string[], label: string, href: string) {
  const safeHref = sanitizeHref(href);
  if (!safeHref) return label;

  const token = `@@INLINE_${tokens.length}@@`;
  tokens.push(
    `<a href="${escapeAttribute(safeHref)}" target="_blank" rel="noreferrer noopener">${label}</a>`,
  );
  return token;
}

function renderInlineMarkdown(text: string) {
  const tokens: string[] = [];
  let html = escapeHtml(text);

  html = html.replace(/`([^`\n]+)`/g, (_, code: string) => {
    const token = `@@INLINE_${tokens.length}@@`;
    tokens.push(`<code>${code}</code>`);
    return token;
  });

  html = html.replace(
    /\[([^\]\n]+)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g,
    (_, label: string, href: string) => {
      return createLinkToken(tokens, label, href);
    },
  );

  html = html.replace(
    /(^|[\s(>])((?:https?:\/\/|mailto:)[^\s<]+)/g,
    (_fullMatch: string, prefix: string, href: string) => {
      return `${prefix}${createLinkToken(tokens, href, href)}`;
    },
  );

  html = html.replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/__([^_\n]+)__/g, "<strong>$1</strong>");
  html = html.replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, "$1<em>$2</em>");
  html = html.replace(/(^|[^_])_([^_\n]+)_(?!_)/g, "$1<em>$2</em>");
  html = html.replace(/~~([^~\n]+)~~/g, "<s>$1</s>");

  return html.replace(/@@INLINE_(\d+)@@/g, (_, index: string) => {
    return tokens[Number(index)] || "";
  });
}

function renderMarkdown(markdown: string) {
  const blocks: string[] = [];
  const source = markdown.replace(/\r\n?/g, "\n");

  for (const part of source.split(/\n{2,}/)) {
    const block = part.trim();
    if (!block) continue;

    const fenceMatch = block.match(/^```([\w-]+)?\n([\s\S]*?)\n```$/);
    if (fenceMatch) {
      const language = fenceMatch[1]?.trim();
      const code = escapeHtml(fenceMatch[2]);
      const className = language
        ? ` class="language-${escapeAttribute(language)}"`
        : "";
      const header = language
        ? `<div class="code-block-header"><span class="code-language">${escapeHtml(formatCodeLanguage(language))}</span></div>`
        : "";
      blocks.push(
        `<div class="code-block">${header}<pre><code${className}>${code}</code></pre></div>`,
      );
      continue;
    }

    const lines = block.split("\n");
    let index = 0;

    while (index < lines.length) {
      const line = lines[index].trim();
      if (!line) {
        index += 1;
        continue;
      }

      if (/^([-*_])(?:\s*\1){2,}$/.test(line.replace(/\s+/g, ""))) {
        blocks.push("<hr>");
        index += 1;
        continue;
      }

      const headingMatch = line.match(/^(#{1,4})\s+(.+)$/);
      if (headingMatch) {
        const level = headingMatch[1].length;
        blocks.push(
          `<h${level}>${renderInlineMarkdown(headingMatch[2].trim())}</h${level}>`,
        );
        index += 1;
        continue;
      }

      if (line.startsWith(">")) {
        const quoteLines: string[] = [];
        while (index < lines.length && lines[index].trim().startsWith(">")) {
          quoteLines.push(
            renderInlineMarkdown(lines[index].trim().replace(/^>\s?/, "")),
          );
          index += 1;
        }
        blocks.push(`<blockquote>${quoteLines.join("<br>")}</blockquote>`);
        continue;
      }

      if (/^\d+\.\s+/.test(line)) {
        const items: string[] = [];
        while (index < lines.length && /^\d+\.\s+/.test(lines[index].trim())) {
          items.push(
            `<li>${renderInlineMarkdown(lines[index].trim().replace(/^\d+\.\s+/, ""))}</li>`,
          );
          index += 1;
        }
        blocks.push(`<ol>${items.join("")}</ol>`);
        continue;
      }

      if (/^[-*+]\s+/.test(line)) {
        const taskItems: string[] = [];
        const items: string[] = [];
        let isTaskList = true;

        while (index < lines.length && /^[-*+]\s+/.test(lines[index].trim())) {
          const content = lines[index].trim().replace(/^[-*+]\s+/, "");
          const taskMatch = content.match(/^\[( |x|X)\]\s+(.*)$/);
          if (!taskMatch) {
            isTaskList = false;
          } else {
            const checked = taskMatch[1].toLowerCase() === "x";
            taskItems.push(
              `<li class="task-list-item"><input class="task-check" type="checkbox" disabled ${checked ? "checked" : ""}><span>${renderInlineMarkdown(taskMatch[2])}</span></li>`,
            );
          }
          items.push(`<li>${renderInlineMarkdown(content)}</li>`);
          index += 1;
        }

        blocks.push(
          isTaskList
            ? `<ul class="task-list">${taskItems.join("")}</ul>`
            : `<ul>${items.join("")}</ul>`,
        );
        continue;
      }

      const paragraphLines: string[] = [];
      while (index < lines.length) {
        const paragraphLine = lines[index].trim();
        if (
          !paragraphLine ||
          /^([-*_])(?:\s*\1){2,}$/.test(paragraphLine.replace(/\s+/g, "")) ||
          /^(#{1,4})\s+(.+)$/.test(paragraphLine) ||
          paragraphLine.startsWith(">") ||
          /^\d+\.\s+/.test(paragraphLine) ||
          /^[-*+]\s+/.test(paragraphLine)
        ) {
          break;
        }
        paragraphLines.push(renderInlineMarkdown(lines[index]));
        index += 1;
      }

      if (paragraphLines.length > 0) {
        blocks.push(`<p>${paragraphLines.join("<br>")}</p>`);
        continue;
      }

      index += 1;
    }
  }

  return blocks.join("");
}

function formatCodeLanguage(language: string) {
  const normalized = language.trim().toLowerCase();
  const labels: Record<string, string> = {
    bash: "Bash",
    css: "CSS",
    html: "HTML",
    javascript: "JavaScript",
    js: "JavaScript",
    json: "JSON",
    markdown: "Markdown",
    md: "Markdown",
    py: "Python",
    python: "Python",
    rs: "Rust",
    rust: "Rust",
    sh: "Shell",
    sql: "SQL",
    ts: "TypeScript",
    tsx: "TSX",
    typescript: "TypeScript",
    xml: "XML",
    yaml: "YAML",
    yml: "YAML",
    jsx: "JSX",
  };

  return labels[normalized] || language;
}

async function saveCurrentWord() {
  if (!currentSelection || !currentResult) return;

  const saveButton = shadowRootRef?.querySelector<HTMLButtonElement>(".save");
  if (saveButton) saveButton.disabled = true;
  setStatus("Saving...");

  const payload: SaveWordPayload = {
    word: currentSelection,
    explain: currentResult,
    example: currentExample,
    wordType: currentWordType,
  };

  try {
    await sendMessage<object>({ type: "saveWord", payload });
    setStatus("Saved.");
    toggleSavePicker(false);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error));
    if (saveButton) saveButton.disabled = false;
  }
}

function startTranslation() {
  activePort?.disconnect();
  syncQuickControls();
  currentResult = "";
  currentReasoning = "";
  reasoningCollapsed = false;
  reasoningAutoCollapsed = false;
  toggleSavePicker(false);
  setTranslateBusy(true);
  setStatus("Translating...");
  setResult("");

  const port = api.runtime.connect({ name: "dictdeck-query" });
  if (!port) {
    setStatus("Extension connection failed.");
    setTranslateBusy(false);
    return;
  }

  activePort = port;
  port.onMessage.addListener((message: unknown) => {
    const response = message as QueryPortResponse;

    switch (response.type) {
      case "start":
        setWordType(response.wordType);
        setStatus("Translating...");
        break;
      case "chunk":
        setStatus("Streaming...");
        setResult(
          currentResult + response.result,
          currentReasoning + (response.reasoning || ""),
        );
        break;
      case "complete":
        setStatus("Ready.");
        setResult(
          response.result || currentResult,
          response.reasoning || currentReasoning,
        );
        setTranslateBusy(false);
        activePort = undefined;
        break;
      case "error":
        setStatus(response.error);
        setResult("");
        setTranslateBusy(false);
        activePort = undefined;
        break;
    }
  });

  port.postMessage({
    type: "translateSelection",
    payload: {
      text: currentSelection,
      title: currentTitle,
      url: currentUrl,
      method: currentMethod,
      dstLang: currentDstLang,
      promptMode: currentPromptMode,
    },
  });
}

function isEditableTarget(target: EventTarget | null) {
  if (!(target instanceof Element)) return false;
  return Boolean(
    target.closest(
      "input, textarea, select, [contenteditable='true'], [contenteditable='']",
    ),
  );
}

async function openPendingSelection() {
  const snapshot = pendingSelection;
  if (!snapshot) return;

  if (snapshot.text.length > MAX_SELECTION_LENGTH) {
    renderFallbackPanel(snapshot, "Selection is too long.");
    return;
  }

  try {
    const settings = await sendMessage<ExtensionSettings>({
      type: "getSettings",
    });
    if (pendingSelection !== snapshot) return;
    if (!settings.baseUrl) {
      renderConfigurationPanel(snapshot);
      return;
    }
    renderPanel(snapshot, settings);
  } catch (error) {
    if (pendingSelection !== snapshot) return;
    renderFallbackPanel(
      snapshot,
      error instanceof Error ? error.message : String(error),
    );
  }
}

function handleSelection(event: Event) {
  if (isPanelEvent(event)) return;
  if (isEditableTarget(event.target)) return;
  if (isPinned) return;

  const selection = window.getSelection();
  const snapshot = selection ? createSelectionSnapshot(selection) : undefined;
  if (!snapshot) {
    closePanel();
    return;
  }

  renderTrigger(snapshot);
}

function renderConfigurationPanel(snapshot: SelectionSnapshot) {
  const root = ensureRoot();
  closePanel();
  currentSelection = snapshot.text;
  currentExample = snapshot.example;
  currentTitle = snapshot.title;
  currentUrl = snapshot.url;

  const panel = document.createElement("section");
  panel.className = "panel";
  panel.innerHTML = `
    <div class="topbar">
      <div class="title">DictDeck</div>
      <button class="icon-button close" type="button" title="Close" aria-label="Close">×</button>
    </div>
    <div class="body">
      <p class="selected"></p>
      <p class="status"><span class="dot"></span><span class="status-text">Please configure the extension first.</span></p>
      <button class="link options" type="button">Open options</button>
    </div>
  `;
  root.append(panel);
  panel.querySelector<HTMLElement>(".selected")!.textContent = currentSelection;
  panel
    .querySelector<HTMLButtonElement>(".close")!
    .addEventListener("click", () => {
      closePanel();
    });
  panel
    .querySelector<HTMLButtonElement>(".options")!
    .addEventListener("click", () => {
      void sendMessage<object>({ type: "openOptions" });
    });
  trapFloatingElementEvents(panel);
  placePanel(panel, snapshot.rect);
}

function renderFallbackPanel(snapshot: SelectionSnapshot, message: string) {
  const root = ensureRoot();
  closePanel();
  currentSelection = snapshot.text;
  currentExample = snapshot.example;
  currentTitle = snapshot.title;
  currentUrl = snapshot.url;

  const panel = document.createElement("section");
  panel.className = "panel";
  panel.innerHTML = `
    <div class="topbar">
      <div class="title">DictDeck</div>
      <button class="icon-button close" type="button" title="Close" aria-label="Close">×</button>
    </div>
    <div class="body">
      <p class="selected"></p>
      <p class="status"><span class="dot"></span><span class="status-text"></span></p>
    </div>
  `;
  root.append(panel);
  panel.querySelector<HTMLElement>(".selected")!.textContent = currentSelection;
  panel.querySelector<HTMLElement>(".status-text")!.textContent = message;
  panel
    .querySelector<HTMLButtonElement>(".close")!
    .addEventListener("click", () => {
      closePanel();
    });
  trapFloatingElementEvents(panel);
  placePanel(panel, snapshot.rect);
}

function scheduleSelection(event: Event) {
  if (isPanelEvent(event)) return;
  if (selectionTimer !== undefined) {
    window.clearTimeout(selectionTimer);
  }
  selectionTimer = window.setTimeout(() => {
    void handleSelection(event);
  }, 180);
}

document.addEventListener("mouseup", scheduleSelection);
document.addEventListener("keyup", scheduleSelection);
document.addEventListener("mousedown", (event) => {
  if (isPanelEvent(event)) return;
  if (isPinned) return;
  closePanel();
});
