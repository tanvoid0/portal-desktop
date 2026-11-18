import {
  getAvailablePackageManagers,
  searchPackages,
  listInstalledPackages,
  getPackageDetails,
  installPackage,
  upgradePackage,
  uninstallPackage,
  checkUpdates,
  getPackageManagerInfo,
  type Package,
  type InstalledPackage,
  type PackageDetails,
  type PackageUpdate,
  type PackageManagerInfo,
} from '../services/packageManagerService';
import { toast } from '$lib/utils/toast';
import { confirmAction } from '$lib/utils/confirm';

function toastError(title: string, err: unknown) {
  toast.error(title, { description: err instanceof Error ? err.message : 'Unknown error' });
}

/** Package-manager selection, search, installed/updates lists and the install/upgrade/uninstall flow. */
export class PackageInstaller {
  loading = $state(true);
  error = $state<string | null>(null);
  availableManagers = $state<string[]>([]);
  selectedManager = $state('');
  managerInfo = $state<PackageManagerInfo | null>(null);

  searchQuery = $state('');
  searchResults = $state<Package[]>([]);
  searching = $state(false);

  installedPackages = $state<InstalledPackage[]>([]);
  loadingInstalled = $state(false);

  availableUpdates = $state<PackageUpdate[]>([]);
  loadingUpdates = $state(false);

  showDetailsDialog = $state(false);
  selectedPackageDetails = $state<PackageDetails | null>(null);
  loadingDetails = $state(false);

  installingPackage = $state<string | null>(null);
  upgradingPackage = $state<string | null>(null);
  uninstallingPackage = $state<string | null>(null);

  private searchTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(private activeTab: () => string) {}

  async loadAvailableManagers() {
    this.loading = true;
    this.error = null;
    try {
      const managers = await getAvailablePackageManagers();
      console.log('Available package managers:', managers);
      this.availableManagers = managers;

      // Auto-select first available manager
      if (managers.length > 0 && !this.selectedManager) {
        this.selectedManager = managers[0];
        console.log('Auto-selected manager:', this.selectedManager);
      }
    } catch (err) {
      this.error = err instanceof Error ? err.message : 'Failed to load package managers';
      console.error('Failed to load package managers:', err);
      toastError('Failed to load package managers', err);
    } finally {
      this.loading = false;
    }
  }

  async loadManagerInfo() {
    if (!this.selectedManager) return;
    try {
      console.log('Loading manager info for:', this.selectedManager);
      const info = await getPackageManagerInfo(this.selectedManager);
      console.log('Manager info loaded:', info);
      this.managerInfo = info;
    } catch (err) {
      console.error('Failed to load manager info:', err);
      toastError('Failed to load manager info', err);
    }
  }

  /** Load the data for the visible tab once the manager is known to be usable. */
  async loadActiveTab() {
    if (!this.managerInfo?.available) return;
    const tab = this.activeTab();
    if (tab === 'installed') await this.loadInstalled();
    else if (tab === 'updates') await this.loadUpdates();
  }

  select(manager: string) {
    this.selectedManager = manager;
    this.searchResults = [];
    this.searchQuery = '';
    this.installedPackages = [];
    this.availableUpdates = [];
    this.managerInfo = null;
  }

  async search() {
    if (!this.selectedManager || !this.searchQuery.trim()) {
      this.searchResults = [];
      return;
    }
    this.searching = true;
    try {
      this.searchResults = await searchPackages(this.selectedManager, this.searchQuery);
    } catch (err) {
      toastError('Search failed', err);
      this.searchResults = [];
    } finally {
      this.searching = false;
    }
  }

  // Debounced search
  onSearchInput() {
    if (this.searchTimer) clearTimeout(this.searchTimer);
    this.searchTimer = setTimeout(() => this.search(), 500);
  }

  async loadInstalled() {
    if (!this.selectedManager) return;
    this.loadingInstalled = true;
    try {
      console.log('Loading installed packages for:', this.selectedManager);
      const packages = await listInstalledPackages(this.selectedManager);
      console.log('Installed packages loaded:', packages.length);
      this.installedPackages = packages;
    } catch (err) {
      console.error('Failed to load installed packages:', err);
      toastError('Failed to load installed packages', err);
      this.installedPackages = [];
    } finally {
      this.loadingInstalled = false;
    }
  }

  async loadUpdates() {
    if (!this.selectedManager) return;
    this.loadingUpdates = true;
    try {
      console.log('Loading updates for:', this.selectedManager);
      const updates = await checkUpdates(this.selectedManager);
      console.log('Updates loaded:', updates.length);
      this.availableUpdates = updates;
    } catch (err) {
      console.error('Failed to check updates:', err);
      toastError('Failed to check updates', err);
      this.availableUpdates = [];
    } finally {
      this.loadingUpdates = false;
    }
  }

  async showDetails(pkg: Package | InstalledPackage) {
    if (!this.selectedManager) return;
    this.loadingDetails = true;
    this.showDetailsDialog = true;
    try {
      this.selectedPackageDetails = await getPackageDetails(this.selectedManager, pkg.id);
    } catch (err) {
      toastError('Failed to load package details', err);
      this.showDetailsDialog = false;
    } finally {
      this.loadingDetails = false;
    }
  }

  async install(pkg: Package) {
    if (!this.selectedManager) return;
    this.installingPackage = pkg.id;
    try {
      await installPackage(this.selectedManager, pkg.id, pkg.version);
      toast.success('Package installed', {
        description: `${pkg.name} has been installed successfully`,
      });
      if (this.activeTab() === 'installed') await this.loadInstalled();
    } catch (err) {
      toastError('Installation failed', err);
    } finally {
      this.installingPackage = null;
    }
  }

  async upgrade(pkg: InstalledPackage | PackageUpdate) {
    if (!this.selectedManager) return;
    this.upgradingPackage = pkg.id;
    try {
      await upgradePackage(this.selectedManager, pkg.id);
      toast.success('Package upgraded', {
        description: `${pkg.name} has been upgraded successfully`,
      });
      await this.loadInstalled();
      await this.loadUpdates();
    } catch (err) {
      toastError('Upgrade failed', err);
    } finally {
      this.upgradingPackage = null;
    }
  }

  async uninstall(pkg: InstalledPackage) {
    if (!this.selectedManager) return;
    const confirmed = await confirmAction(
      `Are you sure you want to uninstall ${pkg.name}?`,
      'Uninstall package'
    );
    if (!confirmed) return;

    this.uninstallingPackage = pkg.id;
    try {
      await uninstallPackage(this.selectedManager, pkg.id);
      toast.success('Package uninstalled', {
        description: `${pkg.name} has been uninstalled successfully`,
      });
      await this.loadInstalled();
    } catch (err) {
      toastError('Uninstall failed', err);
    } finally {
      this.uninstallingPackage = null;
    }
  }
}
