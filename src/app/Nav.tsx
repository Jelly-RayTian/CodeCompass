import { NavLink } from 'react-router-dom';

import { useT } from '@/i18n/useT';

interface NavItem {
  to: string;
  label: string;
  icon: string;
}

export function Nav(): JSX.Element {
  const { t, lang, setLang } = useT();

  const navItems: NavItem[] = [
    { to: '/', label: t.nav.home, icon: '\u2302' },
    { to: '/workspaces', label: t.nav.workspaces, icon: '\u{1F4C1}' },
    { to: '/graph', label: t.nav.graph, icon: '\u{1F578}' },
    { to: '/insights', label: t.nav.insights, icon: '\u{1F4CA}' },
    { to: '/health', label: t.nav.health, icon: '\u{1F3E5}' },
    { to: '/evolution', label: t.nav.evolution, icon: '\u{1F4C8}' },
    { to: '/settings', label: t.nav.settings, icon: '\u2699' },
  ];

  return (
    <nav className="app-nav" aria-label="Main navigation">
      {navItems.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          end={item.to === '/'}
          className={({ isActive }: { isActive: boolean }): string =>
            `nav-item${isActive ? ' active' : ''}`
          }
        >
          <span className="nav-item-icon" aria-hidden="true">
            {item.icon}
          </span>
          <span>{item.label}</span>
        </NavLink>
      ))}

      <button
        className="nav-item lang-toggle"
        onClick={() => setLang(lang === 'en' ? 'zh' : 'en')}
        type="button"
        title={lang === 'en' ? 'Switch to Chinese' : '切换到英文'}
      >
        {lang === 'en' ? '中文' : 'EN'}
      </button>
    </nav>
  );
}
