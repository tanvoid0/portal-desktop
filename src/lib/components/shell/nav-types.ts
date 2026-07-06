export interface NavItem {
  title: string;
  url: string;
  icon?: string;
  description?: string;
  badge?: string | number | null;
  submenu?: NavItem[];
}

export interface NavSection {
  title: string;
  items: NavItem[];
}
