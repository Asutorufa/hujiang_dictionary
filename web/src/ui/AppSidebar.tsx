import {
  BookOpen,
  Clock3,
  Menu,
  MessageSquarePlus,
  PanelLeft,
  Settings2,
  Sparkles,
  Layers3,
  X,
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
  mobileOpen: boolean;
  onToggleMobile: () => void;
  onCloseMobile: () => void;
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
      aria-current={active ? "page" : undefined}
      title={label}
    >
      <span className="app-sidebar-item-icon">{icon}</span>
      <span className="app-sidebar-item-label">{label}</span>
    </button>
  );
}

function BrandMark() {
  return <img className="app-sidebar-brand-mark" src="/dictdeck.svg" alt="" />;
}

export function AppSidebar({
  value,
  onValueChange,
  collapsed,
  onToggleCollapsed,
  mobileOpen,
  onToggleMobile,
  onCloseMobile,
}: AppSidebarProps) {
  const go = (route: string) => {
    onValueChange(route);
    onCloseMobile();
  };

  return (
    <>
      <div className="app-mobile-nav-bar">
        <button
          type="button"
          className="app-mobile-nav-brand"
          onClick={() => go(ROUTE_HOME)}
          aria-label="Go to DictDeck home"
        >
          <BrandMark />
          <span>DictDeck</span>
        </button>
        <button
          type="button"
          className="app-mobile-nav-toggle"
          onClick={onToggleMobile}
          aria-label={mobileOpen ? "Close navigation" : "Open navigation"}
          aria-expanded={mobileOpen}
        >
          {mobileOpen ? <X size={20} /> : <Menu size={20} />}
        </button>
      </div>
      {mobileOpen && (
        <button
          type="button"
          className="app-sidebar-scrim"
          onClick={onCloseMobile}
          aria-label="Close navigation"
        />
      )}
      <aside
        className={`app-sidebar${collapsed ? " is-collapsed" : ""}${mobileOpen ? " is-mobile-open" : ""}`}
        aria-label="Main navigation"
      >
        <div className="app-sidebar-top">
          <div className="app-sidebar-brand-row">
            <button
              type="button"
              className="app-sidebar-brand"
              onClick={() => go(ROUTE_HOME)}
              aria-label="Go to DictDeck home"
            >
              <BrandMark />
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
    </>
  );
}
