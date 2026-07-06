import { invokeClient } from "$lib/utils/invokeClient";

export interface OllamaServiceStatus {
  running: boolean;
  version?: string;
  port?: number;
}

export const ollamaApi = {
  getServiceStatus() {
    return invokeClient.post<OllamaServiceStatus>("get_service_status", {
      sdkType: "ollama",
    });
  },

  getVersions() {
    return invokeClient.post<string[]>("get_ollama_versions");
  },

  getModels() {
    return invokeClient.post<unknown[]>("get_ollama_models");
  },

  getAvailableModels() {
    return invokeClient.post<unknown[]>("get_available_ollama_models");
  },

  installModel(modelName: string) {
    return invokeClient.post("install_ollama_model", { modelName });
  },

  removeModel(modelName: string) {
    return invokeClient.post("remove_ollama_model", { modelName });
  },

  startService() {
    return invokeClient.post("start_service", { sdkType: "ollama" });
  },

  stopService() {
    return invokeClient.post("stop_service", { sdkType: "ollama" });
  },

  checkUpdates() {
    return invokeClient.post("check_ollama_updates");
  },

  update() {
    return invokeClient.post("update_ollama");
  },
};
