import { NavLink } from 'react-router-dom';

interface NavItem {
  to: string;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { to: '/', label: 'Home', icon: '\u2302' },
  { to: '/workspaces', label: 'Workspaces', icon: '\u{1F4C1}' },
  { to: '/settings', label: 'Settings', icon: '\u2699' },
];

export function Nav(): JSX.Element {
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
    </nav>
  );
}
