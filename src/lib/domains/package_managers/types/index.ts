/**
 * Package Managers domain types
 */

export interface PackageManager {
  id: number;
  name: string;
  icon: string;
  icon_type: "devicon" | "file";
  category: string;
  created_at?: string;
  updated_at?: string;
}

export interface SuggestedPackageManager {
  name: string;
  icon: string;
  category: string;
}

export interface PackageManagerGroup {
  category: string;
  package_managers: SuggestedPackageManager[];
}
