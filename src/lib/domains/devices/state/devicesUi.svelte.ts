/**
 * Devices page UI state — just the advanced-mode toggle, persisted so the
 * page doesn't flip back to simple on every visit.
 */

const STORAGE_KEY = 'portal-devices-advanced';

class DevicesUiState {
  advanced = $state(false);

  constructor() {
    if (typeof window === 'undefined') return;
    try {
      this.advanced = localStorage.getItem(STORAGE_KEY) === '1';
    } catch {
      // private mode / blocked storage — simple mode is the right default
    }
  }

  setAdvanced(on: boolean): void {
    this.advanced = on;
    try {
      localStorage.setItem(STORAGE_KEY, on ? '1' : '0');
    } catch {
      // non-persisted is fine
    }
  }
}

export const devicesUi = new DevicesUiState();
