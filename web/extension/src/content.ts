import {
  BUILTIN_METHODS,
  DEFAULT_METHOD,
  LANGUAGES,
  PROMPT_MODES,
  customMethodValue,
  type PromptMode,
} from "@/lib/translation";
import contentAdapterStyles from "@/design-system/content-adapter.css?inline";
import contentPrimitives from "@/design-system/primitives.css?inline";
import designTokens from "@/design-system/tokens.css?inline";
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

const ICONS = {
  pin: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/></svg>',
  close:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>',
  chevron:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg>',
  sparkles:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9.94 14.06 4 20"/><path d="m12 3 1.9 5.8a2 2 0 0 0 1.3 1.3L21 12l-5.8 1.9a2 2 0 0 0-1.3 1.3L12 21l-1.9-5.8a2 2 0 0 0-1.3-1.3L3 12l5.8-1.9a2 2 0 0 0 1.3-1.3z"/></svg>',
  refresh:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>',
  bookmark:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/></svg>',
  brain:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 18V5"/><path d="M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4"/><path d="M17.6 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.6 1.5"/><path d="M17.997 5.125a4 4 0 0 1 2.526 5.77"/><path d="M18 18a4 4 0 0 0 2-7.464"/><path d="M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517"/><path d="M6 18a4 4 0 0 1-2-7.464"/><path d="M6.003 5.125a4 4 0 0 0-2.526 5.77"/></svg>',
  settings:
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>',
};

type StatusState = "idle" | "busy" | "done" | "error";

const RESULT_SKELETON =
  '<div class="result-skeleton" aria-hidden="true"><span class="skeleton-line"></span><span class="skeleton-line"></span><span class="skeleton-line"></span></div>';

function brandMarkup() {
  return `<div class="brand"><img src="${escapeAttribute(api.runtime.getURL("assets/dictdeck.svg"))}" alt="" aria-hidden="true" /><div class="title">DictDeck</div></div>`;
}

/* Detach a floating element and let it fade out instead of vanishing. */
function retireFloating(
  element: Element | null | undefined,
  ghostClass: string,
) {
  if (!(element instanceof HTMLElement)) return;
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    element.remove();
    return;
  }
  element.className = ghostClass;
  element.setAttribute("aria-hidden", "true");
  element.inert = true;
  const remove = () => element.remove();
  element.addEventListener("animationend", remove, { once: true });
  window.setTimeout(remove, 220);
}

function styles() {
  /* Shadow DOM needs the functional :host(...) form for compound selectors;
     `:host:not(...)` / `:host.dark` silently never match. */
  const contentTokenStyles = designTokens
    .replace(/:root:not\(([^)]*)\)/g, ":host(:not($1))")
    .replace(/:root(\.[\w-]+|\[[^\]]+\])/g, ":host($1)")
    .replace(/:root/g, ":host");

  return [
    contentTokenStyles,
    contentAdapterStyles,
    contentPrimitives,
    `
    :host {
      font-family: var(--dd-font-sans);
    }
    .panel,
    .panel-ghost {
      position: fixed;
      z-index: 2147483647;
      width: min(384px, calc(100vw - 24px));
      display: flex;
      flex-direction: column;
      overflow: hidden;
      border: 1px solid color-mix(in srgb, var(--dd-color-border-strong) 60%, transparent);
      border-radius: 14px;
      background: var(--dd-color-surface);
      box-shadow:
        0 0 0 1px rgba(0, 0, 0, 0.02),
        0 2px 6px rgba(0, 0, 0, 0.04),
        0 18px 48px -8px rgba(0, 0, 0, 0.18);
      color: var(--dd-color-text);
      font-size: 13px;
      letter-spacing: 0;
      transform-origin: var(--dd-origin, top center);
    }
    .panel {
      animation: dd-pop-in 240ms var(--dd-ease-out) both;
    }
    .panel-ghost,
    .trigger-ghost {
      pointer-events: none;
      animation: dd-pop-out 140ms ease-in both;
    }
    .panel.is-dragging {
      box-shadow:
        0 0 0 1px rgba(0, 0, 0, 0.02),
        0 4px 10px rgba(0, 0, 0, 0.06),
        0 28px 64px -10px rgba(0, 0, 0, 0.26);
      transition: box-shadow 160ms ease;
    }
    @keyframes dd-pop-out {
      to {
        opacity: 0;
        scale: 0.96;
      }
    }
    @keyframes dd-trigger-in {
      from {
        opacity: 0;
        scale: 0.4;
        translate: 0 6px;
      }
    }
    @keyframes dd-pulse {
      0%, 100% { opacity: 1; scale: 1; }
      50% { opacity: 0.45; scale: 0.8; }
    }
    @keyframes dd-caret {
      50% { opacity: 0; }
    }

    /* Trigger bubble ------------------------------------------------------ */
    .trigger,
    .trigger-ghost {
      position: fixed;
      z-index: 2147483647;
      width: 34px;
      height: 34px;
      display: inline-grid;
      place-items: center;
      border: 1px solid color-mix(in srgb, var(--dd-color-border-strong) 70%, transparent);
      border-radius: 11px;
      background: var(--dd-color-surface);
      box-shadow:
        0 1px 2px rgba(0, 0, 0, 0.06),
        0 8px 24px -4px rgba(0, 0, 0, 0.2);
      cursor: pointer;
      font: inherit;
      padding: 0;
    }
    .trigger {
      animation: dd-trigger-in 280ms var(--dd-ease-spring) both;
      transition:
        scale 180ms var(--dd-ease-spring),
        box-shadow 180ms ease,
        border-color 180ms ease;
    }
    .trigger-mark {
      display: block;
      width: 24px;
      height: 24px;
      margin: 0;
      border-radius: 6px;
      object-fit: contain;
      pointer-events: none;
      user-select: none;
    }
    .trigger:hover {
      scale: 1.08;
      border-color: color-mix(in srgb, var(--dd-color-accent) 45%, var(--dd-color-border));
      box-shadow:
        0 0 0 4px var(--dd-color-accent-soft),
        0 10px 28px -4px rgba(0, 0, 0, 0.24);
    }
    .trigger:active {
      scale: 0.96;
    }
    .trigger:focus-visible {
      outline: 2px solid var(--dd-color-accent);
      outline-offset: 2px;
    }

    /* Top bar ------------------------------------------------------------- */
    .topbar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 8px 8px 0 14px;
      flex: 0 0 auto;
      cursor: grab;
      user-select: none;
    }
    .topbar.dragging {
      cursor: grabbing;
    }
    .brand {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      min-width: 0;
    }
    .brand img {
      width: 18px;
      height: 18px;
      border-radius: 5px;
      flex: 0 0 auto;
      pointer-events: none;
    }
    .title {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      color: var(--dd-color-muted);
      font-size: 12px;
      font-weight: 600;
    }
    .topbar-actions {
      display: inline-flex;
      align-items: center;
      gap: 2px;
      flex: 0 0 auto;
    }
    .icon-button {
      width: 28px;
      height: 28px;
      display: inline-grid;
      place-items: center;
      flex: 0 0 auto;
      border: 0;
      border-radius: 8px;
      background: transparent;
      color: var(--dd-color-muted);
      cursor: pointer;
      font: inherit;
      padding: 0;
      transition:
        background-color 140ms ease,
        color 140ms ease,
        rotate 200ms var(--dd-ease-spring);
    }
    .icon-button svg {
      width: 15px;
      height: 15px;
    }
    .icon-button:hover {
      background: var(--dd-color-surface-muted);
      color: var(--dd-color-text);
    }
    .icon-button:focus-visible {
      outline: none;
      box-shadow: var(--dd-focus-ring);
    }
    .pin-active {
      color: var(--dd-color-accent);
      background: var(--dd-color-accent-soft);
      rotate: -30deg;
    }
    .pin-active:hover {
      color: var(--dd-color-accent-strong);
      background: var(--dd-color-accent-soft);
    }

    /* Body ---------------------------------------------------------------- */
    .body {
      flex: 1 1 auto;
      min-height: 0;
      overflow: auto;
      overscroll-behavior: contain;
      padding: 6px 14px 14px;
      scrollbar-width: thin;
    }
    .selected {
      display: -webkit-box;
      margin: 0 0 12px;
      overflow: hidden;
      color: var(--dd-color-text);
      font-size: 18px;
      font-weight: 700;
      letter-spacing: -0.01em;
      line-height: 1.35;
      white-space: pre-wrap;
      word-break: break-word;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 3;
    }
    .selected.is-long {
      padding-left: 10px;
      border-left: 2px solid var(--dd-color-accent);
      color: var(--dd-color-muted);
      font-size: 13px;
      font-weight: 500;
      line-height: 1.55;
      -webkit-line-clamp: 4;
    }
    .quick-controls {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
      margin: 0 0 12px;
    }
    .quick-controls label {
      position: relative;
      min-width: 0;
      flex: 1 1 0;
    }
    .quick-controls .control-label {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0 0 0 0);
      white-space: nowrap;
    }
    .quick-controls select {
      width: 100%;
      min-width: 0;
      height: 28px;
      border: 1px solid transparent;
      border-radius: 8px;
      background-color: var(--dd-color-surface-muted);
      background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%238a8f89' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m4.5 6.5 3.5 3.5 3.5-3.5'/%3E%3C/svg%3E");
      background-position: right 7px center;
      background-repeat: no-repeat;
      background-size: 12px;
      color: var(--dd-color-text);
      appearance: none;
      cursor: pointer;
      font: inherit;
      font-size: 12px;
      font-weight: 550;
      padding: 0 22px 0 9px;
      text-overflow: ellipsis;
      outline: none;
      transition:
        background-color 140ms ease,
        border-color 140ms ease,
        box-shadow 140ms ease;
    }
    .quick-controls select:hover {
      background-color: color-mix(in srgb, var(--dd-color-surface-muted) 70%, var(--dd-color-border-strong));
    }
    .quick-controls select:focus-visible {
      border-color: var(--dd-color-accent);
      box-shadow: var(--dd-focus-ring);
    }

    /* Status -------------------------------------------------------------- */
    .status {
      display: flex;
      align-items: center;
      gap: 7px;
      margin: 0 0 10px;
      color: var(--dd-color-muted);
      font-size: 12px;
      line-height: 1.4;
    }
    .status-text {
      animation: dd-fade-in 180ms ease;
    }
    .dot {
      width: 6px;
      height: 6px;
      flex: 0 0 auto;
      border-radius: 999px;
      background: var(--dd-color-border-strong);
      transition: background-color 200ms ease;
    }
    .status[data-state="busy"] .dot {
      background: var(--dd-color-accent);
      animation: dd-pulse 1s ease-in-out infinite;
    }
    .status[data-state="done"] .dot {
      background: var(--dd-color-positive);
    }
    .status[data-state="error"] {
      color: var(--dd-color-danger);
    }
    .status[data-state="error"] .dot {
      background: var(--dd-color-danger);
    }

    /* Result -------------------------------------------------------------- */
    .result {
      margin: 0;
      color: var(--dd-color-text);
      font-size: 14px;
      line-height: 1.65;
      word-break: break-word;
    }
    .result.is-entering {
      animation: dd-fade-up 280ms var(--dd-ease-out) both;
    }
    .result.is-streaming > :last-child::after {
      display: inline-block;
      width: 2px;
      height: 1em;
      margin-left: 2px;
      vertical-align: -0.15em;
      background: var(--dd-color-accent);
      content: "";
      animation: dd-caret 900ms steps(1) infinite;
    }
    .result-skeleton {
      display: grid;
      gap: 9px;
      padding-top: 4px;
    }
    .result-skeleton .skeleton-line:nth-child(2) { width: 88%; }
    .result-skeleton .skeleton-line:nth-child(3) { width: 60%; }
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
    .markdown li::marker {
      color: var(--dd-color-muted);
    }
    .markdown li + li {
      margin-top: 4px;
    }
    .markdown a {
      color: var(--dd-color-accent-strong);
      text-decoration: underline;
      text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
      text-underline-offset: 3px;
      transition: text-decoration-color 140ms ease;
      cursor: pointer;
    }
    .markdown a:hover {
      text-decoration-color: currentColor;
    }
    .markdown a:focus-visible {
      outline: 2px solid var(--dd-color-accent);
      outline-offset: 2px;
      border-radius: 4px;
    }
    .markdown strong {
      font-weight: 700;
    }
    .markdown em {
      font-style: italic;
    }
    .markdown code {
      padding: 1px 5px;
      border-radius: 5px;
      background: var(--dd-color-surface-muted);
      font-family: var(--dd-font-mono);
      font-size: 0.88em;
    }
    .markdown pre {
      padding: 10px 12px;
      border-radius: 9px;
      background: var(--dd-color-surface-muted);
      overflow: auto;
      tab-size: 2;
    }
    .markdown pre code {
      display: block;
      padding: 0;
      background: transparent;
      white-space: pre;
      line-height: 1.6;
    }
    .markdown .code-block {
      margin: 0 0 10px;
      border: 1px solid var(--dd-color-border);
      border-radius: 9px;
      overflow: hidden;
      background: var(--dd-color-surface-muted);
    }
    .markdown .code-block-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
      min-height: 28px;
      padding: 0 12px;
      border-bottom: 1px solid var(--dd-color-border);
    }
    .markdown .code-language {
      color: var(--dd-color-muted);
      font-size: 11px;
      font-weight: 600;
    }
    .markdown .code-block pre {
      margin: 0;
      border-radius: 0;
      background: transparent;
    }
    .markdown blockquote {
      padding: 2px 0 2px 12px;
      border-left: 2px solid var(--dd-color-border-strong);
      color: var(--dd-color-muted);
    }
    .markdown h1,
    .markdown h2,
    .markdown h3,
    .markdown h4 {
      color: var(--dd-color-text);
      font-weight: 700;
      letter-spacing: -0.01em;
      line-height: 1.35;
    }
    .markdown h1 {
      font-size: 17px;
    }
    .markdown h2 {
      font-size: 15px;
    }
    .markdown h3 {
      font-size: 14px;
    }
    .markdown h4 {
      font-size: 13px;
      color: var(--dd-color-muted);
    }
    .markdown hr {
      height: 1px;
      margin: 14px 0;
      border: 0;
      background: var(--dd-color-border);
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
      accent-color: var(--dd-color-accent);
      pointer-events: none;
    }
    .markdown-empty {
      color: var(--dd-color-muted);
      font-size: 13px;
    }

    /* Reasoning ----------------------------------------------------------- */
    .reasoning-wrap {
      margin: 0 0 12px;
      border-radius: 10px;
      background: var(--dd-color-surface-muted);
      overflow: hidden;
    }
    .reasoning-wrap[hidden] {
      display: none;
    }
    .reasoning-toggle {
      width: 100%;
      display: flex;
      align-items: center;
      gap: 6px;
      min-height: 32px;
      border: 0;
      background: transparent;
      color: var(--dd-color-muted);
      font: inherit;
      font-size: 12px;
      font-weight: 600;
      padding: 0 10px;
      cursor: pointer;
    }
    .reasoning-toggle:hover {
      color: var(--dd-color-text);
    }
    .reasoning-toggle svg {
      width: 13px;
      height: 13px;
    }
    .reasoning-chevron {
      display: inline-grid;
      margin-left: auto;
      transition: rotate 220ms var(--dd-ease-out);
    }
    .reasoning-wrap.collapsed .reasoning-chevron {
      rotate: -90deg;
    }
    .reasoning-body {
      display: grid;
      grid-template-rows: 1fr;
      transition: grid-template-rows 260ms var(--dd-ease-out);
    }
    .reasoning-wrap.collapsed .reasoning-body {
      grid-template-rows: 0fr;
    }
    .reasoning {
      min-height: 0;
      margin: 0;
      overflow: hidden;
      padding: 0 10px;
      color: var(--dd-color-muted);
      font-size: 12px;
      line-height: 1.55;
      word-break: break-word;
      opacity: 1;
      transition: opacity 200ms ease, padding 260ms var(--dd-ease-out);
    }
    .reasoning-wrap:not(.collapsed) .reasoning {
      padding-bottom: 10px;
    }
    .reasoning-wrap.collapsed .reasoning {
      opacity: 0;
    }

    /* Footer -------------------------------------------------------------- */
    .footer {
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      min-height: 50px;
      padding: 9px 12px;
      flex: 0 0 auto;
      border-top: 1px solid var(--dd-color-border);
      background: var(--dd-color-surface);
    }
    .footer-actions {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      flex: 0 0 auto;
      animation: dd-fade-in 160ms ease;
    }
    .save-picker.visible ~ .footer-actions {
      display: none;
    }
    .segment {
      display: inline-flex;
      gap: 2px;
      padding: 2px;
      border-radius: 9px;
      background: var(--dd-color-surface-muted);
    }
    .segment button,
    .primary,
    .secondary,
    .link {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      min-height: 30px;
      border: 0;
      font: inherit;
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
      white-space: nowrap;
      transition:
        background-color 140ms ease,
        color 140ms ease,
        box-shadow 140ms ease,
        opacity 140ms ease,
        scale 120ms ease;
    }
    .primary svg,
    .secondary svg,
    .link svg {
      width: 14px;
      height: 14px;
      flex: 0 0 auto;
    }
    .primary:active:not(:disabled),
    .secondary:active:not(:disabled),
    .segment button:active {
      scale: 0.97;
    }
    .segment button {
      min-height: 26px;
      padding: 0 10px;
      border-radius: 7px;
      background: transparent;
      color: var(--dd-color-muted);
    }
    .segment button:hover {
      background: var(--dd-color-surface);
      color: var(--dd-color-text);
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
    }
    .segment button.active {
      background: var(--dd-color-surface);
      color: var(--dd-color-text);
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
    }
    .primary {
      padding: 0 12px;
      border-radius: 8px;
      background: var(--dd-color-accent);
      color: #ffffff;
      box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.16),
        0 1px 2px color-mix(in srgb, var(--dd-color-accent) 35%, transparent);
    }
    .primary:hover:not(:disabled) {
      background: var(--dd-color-accent-strong);
    }
    .primary:disabled,
    .secondary:disabled {
      cursor: not-allowed;
      opacity: 0.45;
      box-shadow: none;
    }
    .secondary {
      padding: 0 11px;
      border: 1px solid var(--dd-color-border);
      border-radius: 8px;
      background: var(--dd-color-surface);
      color: var(--dd-color-text);
    }
    .secondary:hover:not(:disabled) {
      background: var(--dd-color-surface-muted);
    }
    .translate.is-busy svg {
      animation: dd-spin 900ms linear infinite;
    }
    .cancel-save {
      border-color: transparent;
      color: var(--dd-color-muted);
    }
    .save-picker {
      display: none;
      align-items: center;
      gap: 8px;
      margin-right: auto;
    }
    .save-picker.visible {
      display: inline-flex;
      width: 100%;
      animation: dd-fade-up 200ms var(--dd-ease-out);
    }
    .save-picker .cancel-save {
      margin-left: auto;
    }
    .save-picker-label {
      color: var(--dd-color-muted);
      font-size: 12px;
      font-weight: 600;
    }
    .link {
      min-height: 32px;
      padding: 0 12px;
      border-radius: 8px;
      background: var(--dd-color-accent-soft);
      color: var(--dd-color-accent-strong);
    }
    .link:hover {
      background: color-mix(in srgb, var(--dd-color-accent) 18%, transparent);
    }
    `,
  ].join("\n");
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
  cancelResultRender();
  if (resetPinned) {
    isPinned = false;
  }
  retireFloating(shadowRootRef?.querySelector(".panel"), "panel-ghost");
  retireFloating(shadowRootRef?.querySelector(".trigger"), "trigger-ghost");
}

function isPanelEvent(event: Event) {
  if (!rootHost) return false;
  return event.composedPath().includes(rootHost);
}

function setStatus(text: string, state: StatusState = "idle") {
  const status = shadowRootRef?.querySelector<HTMLElement>(".status");
  const statusText = status?.querySelector<HTMLElement>(".status-text");
  if (!status || !statusText) return;
  status.dataset.state = state;
  if (statusText.textContent === text) return;
  /* Re-insert the node so the fade-in animation replays on each change. */
  const next = statusText.cloneNode() as HTMLElement;
  next.textContent = text;
  statusText.replaceWith(next);
}

function isTranslating() {
  return Boolean(
    shadowRootRef?.querySelector(".translate")?.classList.contains("is-busy"),
  );
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

let resultRenderFrame: number | undefined;
let renderedResult: string | undefined;
let renderedReasoning: string | undefined;

/* Streaming chunks can arrive many times per frame. Keep the state current
   immediately, but re-render the Markdown at most once per animation frame. */
function setResult(
  result: string,
  reasoning?: string,
  options: { defer?: boolean } = {},
) {
  currentResult = result;
  currentReasoning = reasoning || "";

  if (options.defer) {
    if (resultRenderFrame === undefined) {
      resultRenderFrame = window.requestAnimationFrame(flushResult);
    }
    return;
  }

  flushResult();
}

function cancelResultRender() {
  if (resultRenderFrame !== undefined) {
    window.cancelAnimationFrame(resultRenderFrame);
    resultRenderFrame = undefined;
  }
  renderedResult = undefined;
  renderedReasoning = undefined;
}

function flushResult() {
  if (resultRenderFrame !== undefined) {
    window.cancelAnimationFrame(resultRenderFrame);
    resultRenderFrame = undefined;
  }

  const previousResult = renderedResult || "";
  const resultEl = shadowRootRef?.querySelector<HTMLElement>(".result");
  const reasoningWrap =
    shadowRootRef?.querySelector<HTMLElement>(".reasoning-wrap");
  const reasoningEl = shadowRootRef?.querySelector<HTMLElement>(".reasoning");
  const saveButton = shadowRootRef?.querySelector<HTMLButtonElement>(".save");

  if (resultEl && renderedResult !== currentResult) {
    const busy = isTranslating();
    resultEl.innerHTML = currentResult
      ? renderMarkdown(currentResult)
      : busy
        ? RESULT_SKELETON
        : '<p class="markdown-empty">Press Translate to look this up.</p>';
    resultEl.classList.toggle("is-streaming", busy && Boolean(currentResult));
    if (!previousResult && currentResult) {
      resultEl.classList.remove("is-entering");
      void resultEl.offsetWidth;
      resultEl.classList.add("is-entering");
    }
  }
  if (reasoningWrap) {
    reasoningWrap.hidden = !currentReasoning;
  }
  if (reasoningEl && renderedReasoning !== currentReasoning) {
    reasoningEl.innerHTML = renderMarkdown(currentReasoning);
  }
  renderedResult = currentResult;
  renderedReasoning = currentReasoning;

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
  button.classList.toggle("is-busy", isBusy);
  /* Force the next flush to swap between skeleton and placeholder. */
  renderedResult = undefined;
  button.innerHTML = `${isBusy ? ICONS.refresh : ICONS.sparkles}<span>${isBusy ? "Translating" : "Translate"}</span>`;
  if (!isBusy) {
    shadowRootRef?.querySelector(".result")?.classList.remove("is-streaming");
  }
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
  if (!manualPosition) {
    const originX = Math.round(rect.left + rect.width / 2 - finalLeft);
    panel.style.setProperty(
      "--dd-origin",
      `${originX}px ${shouldPlaceBelow ? "top" : "bottom"}`,
    );
  }
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
        panel.classList.add("is-dragging");
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
      panel.classList.remove("is-dragging");
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

function renderSelected(panel: HTMLElement) {
  const selected = panel.querySelector<HTMLElement>(".selected");
  if (!selected) return;
  selected.textContent = currentSelection;
  selected.classList.toggle(
    "is-long",
    currentSelection.length > 48 || /\n/.test(currentSelection),
  );
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
  const mark = document.createElement("img");
  mark.className = "trigger-mark";
  mark.src = api.runtime.getURL("assets/dictdeck.svg");
  mark.alt = "";
  mark.setAttribute("aria-hidden", "true");
  trigger.append(mark);
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
      ${brandMarkup()}
      <div class="topbar-actions">
        <button class="icon-button pin" type="button" title="Pin" aria-label="Pin" aria-pressed="false">${ICONS.pin}</button>
        <button class="icon-button close" type="button" title="Close" aria-label="Close">${ICONS.close}</button>
      </div>
    </div>
    <div class="body">
      <p class="selected"></p>
      <div class="quick-controls">
        <label title="Method">
          <span class="control-label">Method</span>
          <select class="method">${methodOptions(settings)}</select>
        </label>
        <label title="Target">
          <span class="control-label">Target</span>
          <select class="dst-lang">${languageOptions(currentDstLang)}</select>
        </label>
        <label title="Prompt">
          <span class="control-label">Prompt</span>
          <select class="prompt-mode">${promptModeOptions(currentPromptMode)}</select>
        </label>
      </div>
      <p class="status" data-state="idle"><span class="dot"></span><span class="status-text">Ready</span></p>
      <div class="reasoning-wrap collapsed" hidden>
        <button class="reasoning-toggle" type="button" aria-expanded="false">
          ${ICONS.brain}
          <span>Thinking</span>
          <span class="reasoning-chevron">${ICONS.chevron}</span>
        </button>
        <div class="reasoning-body"><div class="reasoning markdown"></div></div>
      </div>
      <div class="result markdown"></div>
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
        <button class="secondary translate" type="button">${ICONS.sparkles}<span>Translate</span></button>
        <button class="primary save" type="button" disabled>${ICONS.bookmark}<span>Save</span></button>
      </div>
    </div>
  `;

  root.append(panel);
  renderSelected(panel);
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
  setStatus("Saving…", "busy");

  const payload: SaveWordPayload = {
    word: currentSelection,
    explain: currentResult,
    example: currentExample,
    wordType: currentWordType,
  };

  try {
    await sendMessage<object>({ type: "saveWord", payload });
    setStatus("Saved to your vocabulary", "done");
    toggleSavePicker(false);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), "error");
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
  setStatus("Translating…", "busy");
  setResult("");

  const port = api.runtime.connect({ name: "dictdeck-query" });
  if (!port) {
    setStatus("Extension connection failed.", "error");
    setTranslateBusy(false);
    return;
  }

  activePort = port;
  port.onMessage.addListener((message: unknown) => {
    const response = message as QueryPortResponse;

    switch (response.type) {
      case "start":
        setWordType(response.wordType);
        setStatus("Translating…", "busy");
        break;
      case "chunk":
        setStatus("Writing…", "busy");
        setResult(
          currentResult + response.result,
          currentReasoning + (response.reasoning || ""),
          { defer: true },
        );
        break;
      case "complete":
        setTranslateBusy(false);
        setStatus("Done", "done");
        setResult(
          response.result || currentResult,
          response.reasoning || currentReasoning,
        );
        activePort = undefined;
        break;
      case "error":
        setTranslateBusy(false);
        setStatus(response.error, "error");
        setResult("");
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
      ${brandMarkup()}
      <button class="icon-button close" type="button" title="Close" aria-label="Close">${ICONS.close}</button>
    </div>
    <div class="body">
      <p class="selected"></p>
      <p class="status" data-state="idle"><span class="dot"></span><span class="status-text">Connect a DictDeck server to start translating.</span></p>
      <button class="link options" type="button">${ICONS.settings}<span>Open settings</span></button>
    </div>
  `;
  root.append(panel);
  renderSelected(panel);
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
      ${brandMarkup()}
      <button class="icon-button close" type="button" title="Close" aria-label="Close">${ICONS.close}</button>
    </div>
    <div class="body">
      <p class="selected"></p>
      <p class="status" data-state="error"><span class="dot"></span><span class="status-text"></span></p>
    </div>
  `;
  root.append(panel);
  renderSelected(panel);
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
