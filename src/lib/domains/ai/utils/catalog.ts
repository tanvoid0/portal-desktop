import type { CatalogModel } from "../types/index.js";

/** Human-readable summary of per-model metadata from the catalog. */
export function formatModelMetadata(model: CatalogModel): string {
  const meta = model.metadata ?? {};
  const parts: string[] = [];

  if (model.source === "alias" && model.backend_id && model.backend_id !== model.id) {
    parts.push(`→ ${model.backend_id}`);
  }

  const parameterSize = meta.parameter_size;
  if (typeof parameterSize === "string" && parameterSize) {
    parts.push(parameterSize);
  }

  const family = meta.family;
  if (typeof family === "string" && family) {
    parts.push(family);
  }

  const displayName = meta.display_name;
  if (typeof displayName === "string" && displayName && displayName !== model.id) {
    parts.push(displayName);
  }

  const quantization = meta.quantization_level;
  if (typeof quantization === "string" && quantization) {
    parts.push(quantization);
  }

  const size = meta.size;
  if (typeof size === "number" && size > 0) {
    parts.push(formatBytes(size));
  }

  if (model.source === "alias") {
    parts.push("alias");
  } else if (model.source === "live") {
    parts.push("live");
  }

  return parts.join(" · ");
}

/** Label for model selectors: id plus optional metadata suffix. */
export function formatModelLabel(model: CatalogModel): string {
  const summary = formatModelMetadata(model);
  return summary ? `${model.id} (${summary})` : model.id;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/** Flatten catalog providers into a deduplicated model list (by id). */
export function flattenCatalogModels(
  providers: Array<{ models: CatalogModel[] }>,
): CatalogModel[] {
  const byId = new Map<string, CatalogModel>();
  for (const provider of providers) {
    for (const model of provider.models) {
      if (!byId.has(model.id)) {
        byId.set(model.id, model);
      }
    }
  }
  return Array.from(byId.values()).sort((a, b) => a.id.localeCompare(b.id));
}
