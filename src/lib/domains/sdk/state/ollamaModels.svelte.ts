import { ollamaApi } from '../api/ollamaApi';
import { logger } from '$lib/domains/shared';
import { toast } from '$lib/utils/toast';

const log = logger.createScoped('OllamaService');

/** Local + library model lists and the install/remove flow for the Ollama page. */
export class OllamaModels {
  models = $state<any[]>([]);
  availableModels = $state<Record<string, any[]>>({});
  loading = $state(false);
  error = $state<string | null>(null);
  tab = $state<'local' | 'library'>('local');

  installingModel = $state<string | null>(null);
  installationProgress = $state(0);
  installationStatus = $state('');

  constructor(private isRunning: () => boolean) {}

  async loadLocal() {
    try {
      log.debug('Loading local models', { serviceRunning: this.isRunning() });
      this.loading = true;
      this.error = null;

      if (!this.isRunning()) {
        this.error = 'Ollama service is not running. Please start the service first.';
        toast.warning(
          'Service Not Running',
          'Please start the Ollama service before managing models.'
        );
        return;
      }

      const modelsData = (await ollamaApi.getModels()) as any[];
      // Backend reports family "Unknown" when /api/tags omits it; don't show it as a badge
      this.models = modelsData.map((m) =>
        m.family === 'Unknown' ? { ...m, family: undefined } : m
      );
      log.info('Ollama models loaded', { count: modelsData.length });
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to load models';
      this.error = msg;
      log.error('Failed to load Ollama models', error);
      if (msg.includes('not installed') || msg.includes('program not found')) {
        toast.error('Ollama Not Installed', 'Please install Ollama first to manage models.');
      } else {
        toast.error('Failed to Load Models', msg);
      }
    } finally {
      this.loading = false;
    }
  }

  async loadLibrary() {
    try {
      log.debug('Loading available models from library');
      this.loading = true;
      this.error = null;
      // The library is online, so it loads without the service running.
      const data = await ollamaApi.getAvailableModels();
      this.availableModels = data;
      log.info('Available Ollama models loaded', {
        categories: Object.keys(data).length,
        totalModels: Object.values(data).flat().length,
      });
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to load available models';
      this.error = msg;
      log.error('Failed to load available Ollama models', error);
      toast.error('Failed to Load Available Models', msg);
    } finally {
      this.loading = false;
    }
  }

  /** Reload whichever list the active sub-tab shows. */
  reload() {
    return this.tab === 'local' ? this.loadLocal() : this.loadLibrary();
  }

  show(tab: 'local' | 'library') {
    this.tab = tab;
    return this.reload();
  }

  async install(modelName: string) {
    try {
      if (!this.isRunning()) {
        toast.warning(
          'Service Not Running',
          'Please start the Ollama service before installing models.'
        );
        return;
      }

      this.installingModel = modelName;
      this.installationProgress = 50;
      log.info('Starting model installation', { modelName });
      toast.info('Download Started', `Starting download of ${modelName}...`);
      // Honest status - this is a long-running operation
      this.installationStatus = `Downloading ${modelName}... This may take several minutes for large models`;

      const timeout = new Promise((_, reject) =>
        setTimeout(
          () =>
            reject(
              new Error('Installation timeout - this may take several minutes for large models')
            ),
          300000 // 5 minutes
        )
      );
      const result = await Promise.race([ollamaApi.installModel(modelName), timeout]);

      this.installationProgress = 100;
      this.installationStatus = 'Installation complete!';
      await this.loadLocal();
      log.info('Model installed successfully', { modelName, result });
      toast.success('Model Installed', `${modelName} has been installed successfully`);
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      log.error('Failed to install model', { modelName, error: msg });
      toast.error('Installation Failed', `Failed to install ${modelName}: ${msg}`);
    } finally {
      // Keep progress visible for 2 seconds after completion
      setTimeout(() => this.resetProgress(), 2000);
    }
  }

  cancelInstall() {
    if (!this.installingModel) return;
    const modelName = this.installingModel;
    log.info('Cancelling installation', { modelName });
    this.resetProgress();
    toast.info(
      'Installation Cancelled',
      `Installation of ${modelName} has been cancelled. Note: The download may continue in the background.`
    );
  }

  async remove(modelName: string) {
    try {
      if (!this.isRunning()) {
        toast.warning(
          'Service Not Running',
          'Please start the Ollama service before removing models.'
        );
        return;
      }

      log.info('Removing model:', modelName);
      const result = await ollamaApi.removeModel(modelName);
      await this.loadLocal();
      log.info('Model removed successfully', { modelName, result });
      toast.success('Model Removed', `${modelName} has been removed successfully`);
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      log.error('Failed to remove model', { modelName, error: msg });
      toast.error('Removal Failed', `Failed to remove ${modelName}: ${msg}`);
    }
  }

  private resetProgress() {
    this.installingModel = null;
    this.installationProgress = 0;
    this.installationStatus = '';
  }
}
