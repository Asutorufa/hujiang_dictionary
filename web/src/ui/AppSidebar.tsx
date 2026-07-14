import {
  BookOpen,
  Clock3,
  Library,
  MessageSquarePlus,
  Search,
  Settings2,
  Sparkles,
  Layers3,
  PanelLeft,
} from "lucide-react";
import { ReactNode } from "react";
import {
  ROUTE_CONFIG,
  ROUTE_FLASHCARD,
  ROUTE_HOME,
  ROUTE_WORDS,
} from "@/lib/constants";

type AppSidebarProps = {
  value: string;
  onValueChange: (value: string) => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
};

type SidebarItemProps = {
  icon: ReactNode;
  label: string;
  active?: boolean;
  onClick: () => void;
};

function SidebarItem({ icon, label, active, onClick }: SidebarItemProps) {
  return (
    <button
      type="button"
      className={`app-sidebar-item${active ? " is-active" : ""}`}
      onClick={onClick}
    >
      <span className="app-sidebar-item-icon">{icon}</span>
      <span className="app-sidebar-item-label">{label}</span>
    </button>
  );
}

export function AppSidebar({ value, onValueChange, collapsed, onToggleCollapsed }: AppSidebarProps) {
  const go = (route: string) => onValueChange(route);

  return (
    <aside className={`app-sidebar${collapsed ? " is-collapsed" : ""}`} aria-label="Main navigation">
      <div className="app-sidebar-top">
        <div className="app-sidebar-brand-row">
          <button
            type="button"
            className="app-sidebar-brand"
            onClick={() => go(ROUTE_HOME)}
            aria-label="Go to DictDeck home"
          >
            <span className="app-sidebar-brand-mark">D</span>
            <span>DictDeck</span>
          </button>
          <button
            type="button"
            className="app-sidebar-collapse"
            onClick={onToggleCollapsed}
            aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
            aria-expanded={!collapsed}
            title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            <PanelLeft size={18} />
          </button>
        </div>

        <button
          type="button"
          className="app-sidebar-new"
          onClick={() => go(ROUTE_HOME)}
        >
          <MessageSquarePlus size={19} />
          <span>New lookup</span>
          <span className="app-sidebar-shortcut">⌘K</span>
        </button>

        <div className="app-sidebar-utility-list">
          <button type="button" className="app-sidebar-utility" onClick={() => go(ROUTE_WORDS)}>
            <Search size={18} />
            <span>Search vocabulary</span>
          </button>
          <button type="button" className="app-sidebar-utility" onClick={() => go(ROUTE_WORDS)}>
            <Library size={18} />
            <span>Library</span>
          </button>
        </div>

        <div className="app-sidebar-section-label">Workspace</div>
        <nav className="app-sidebar-nav">
          <SidebarItem
            icon={<Sparkles size={18} />}
            label="Translate"
            active={value === ROUTE_HOME}
            onClick={() => go(ROUTE_HOME)}
          />
          <SidebarItem
            icon={<BookOpen size={18} />}
            label="Vocabulary"
            active={value === ROUTE_WORDS}
            onClick={() => go(ROUTE_WORDS)}
          />
          <SidebarItem
            icon={<Layers3 size={18} />}
            label="Daily review"
            active={value === ROUTE_FLASHCARD}
            onClick={() => go(ROUTE_FLASHCARD)}
          />
          <SidebarItem
            icon={<Settings2 size={18} />}
            label="Settings"
            active={value === ROUTE_CONFIG}
            onClick={() => go(ROUTE_CONFIG)}
          />
        </nav>

        <div className="app-sidebar-section-label app-sidebar-recent-label">Recent</div>
        <div className="app-sidebar-recent-list">
          <button type="button" onClick={() => go(ROUTE_HOME)}>Translate a word</button>
          <button type="button" onClick={() => go(ROUTE_WORDS)}>Saved vocabulary</button>
          <button type="button" onClick={() => go(ROUTE_FLASHCARD)}>Review session</button>
        </div>
      </div>

      <div className="app-sidebar-footer">
        <div className="app-sidebar-footer-avatar">H</div>
        <div className="app-sidebar-footer-copy">
          <strong>Hujiang workspace</strong>
          <span>Local dictionary</span>
        </div>
        <Clock3 size={16} className="app-sidebar-footer-icon" />
      </div>
    </aside>
  );
}
