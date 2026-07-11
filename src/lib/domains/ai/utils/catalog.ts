import type { CatalogModel, CatalogModelCapabilities, CatalogProvider } from "../types/index.js";

/** Local backends first, then cloud (matches agent-platform config UI). */
export const CATALOG_PROVIDER_ORDER = [
  "ollama",
  "lm_studio",
  "gemini",
  "anthropic",
  "aimlapi",
] as const;

export function sortCatalogProviders(
  providers: CatalogProvider[],
): CatalogProvider[] {
  const order = new Map(
    CATALOG_PROVIDER_ORDER.map((id, index) => [id, index]),
  );
  return [...providers].sort((a, b) => {
    const aOrder = order.get(a.id as (typeof CATALOG_PROVIDER_ORDER)[number]);
    const bOrder = order.get(b.id as (typeof CATALOG_PROVIDER_ORDER)[number]);
    const aRank = aOrder ?? 99;
    const bRank = bOrder ?? 99;
    if (aRank !== bRank) return aRank - bRank;
    return a.label.localeCompare(b.label);
  });
}

/** Providers that are configured and have at least one catalog model. */
export function selectableCatalogProviders(
  providers: CatalogProvider[],
): CatalogProvider[] {
  return sortCatalogProviders(providers).filter(
    (provider) =>
      provider.configured &&
      provider.reachable !== false &&
      provider.models.length > 0,
  );
}

function withProviderId(
  models: CatalogModel[],
  providerId: string,
): CatalogModel[] {
  return models.map((model) => ({
    ...model,
    provider: model.provider ?? providerId,
  }));
}

export function modelsForCatalogProvider(
  providers: CatalogProvider[],
  providerId: string,
): CatalogModel[] {
  const provider = providers.find((entry) => entry.id === providerId);
  if (!provider) return [];
  return withProviderId(provider.models, providerId).sort((a, b) =>
    a.id.localeCompare(b.id),
  );
}

export function defaultModelForCatalogProvider(
  provider: CatalogProvider | undefined,
  resolvedDefault?: { provider: string; model: string } | null,
): string | null {
  if (!provider) return null;
  if (resolvedDefault?.provider === provider.id && resolvedDefault.model) {
    return resolvedDefault.model;
  }
  if (provider.default_model) return provider.default_model;
  return provider.models[0]?.id ?? null;
}

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

export interface ModelDisplayBadge {
  key: string;
  title: string;
  label: string;
  group: "spec" | "status";
  className: string;
  valueClassName: string;
}

export interface ModelDisplayParts {
  id: string;
  provider?: string;
  backendId?: string;
  displayName?: string;
  specBadges: ModelDisplayBadge[];
  statusBadges: ModelDisplayBadge[];
}

const MODEL_BADGE_STYLES = {
  params: {
    badge:
      "border-sky-200/80 bg-sky-50 dark:border-sky-800 dark:bg-sky-950/60",
    value: "text-sky-700 dark:text-sky-300",
  },
  family: {
    badge:
      "border-violet-200/80 bg-violet-50 dark:border-violet-800 dark:bg-violet-950/60",
    value: "text-violet-700 dark:text-violet-300",
  },
  quantization: {
    badge:
      "border-amber-200/80 bg-amber-50 dark:border-amber-800 dark:bg-amber-950/60",
    value: "text-amber-800 dark:text-amber-300",
  },
  size: {
    badge:
      "border-slate-200/80 bg-slate-50 dark:border-slate-700 dark:bg-slate-900/60",
    value: "text-slate-600 dark:text-slate-300",
  },
  provider: {
    badge:
      "border-indigo-200/80 bg-indigo-50 dark:border-indigo-800 dark:bg-indigo-950/60",
    value: "text-indigo-700 dark:text-indigo-300",
  },
  live: {
    badge:
      "border-emerald-200/80 bg-emerald-50 dark:border-emerald-800 dark:bg-emerald-950/60",
    value: "text-emerald-700 dark:text-emerald-300",
  },
  alias: {
    badge:
      "border-orange-200/80 bg-orange-50 dark:border-orange-800 dark:bg-orange-950/60",
    value: "text-orange-700 dark:text-orange-300",
  },
  extra: {
    badge:
      "border-zinc-200/80 bg-zinc-50 dark:border-zinc-700 dark:bg-zinc-900/60",
    value: "text-zinc-600 dark:text-zinc-300",
  },
  toolsYes: {
    badge:
      "border-emerald-200/80 bg-emerald-50 dark:border-emerald-800 dark:bg-emerald-950/60",
    value: "text-emerald-700 dark:text-emerald-300",
  },
  toolsNo: {
    badge:
      "border-rose-200/80 bg-rose-50 dark:border-rose-800 dark:bg-rose-950/60",
    value: "text-rose-700 dark:text-rose-300",
  },
  vision: {
    badge:
      "border-cyan-200/80 bg-cyan-50 dark:border-cyan-800 dark:bg-cyan-950/60",
    value: "text-cyan-700 dark:text-cyan-300",
  },
} as const;

function capabilityBadges(
  caps: CatalogModelCapabilities | undefined,
): ModelDisplayBadge[] {
  if (!caps) return [];
  const out: ModelDisplayBadge[] = [];
  if (caps.tools === true) {
    out.push(makeBadge("tools", "Tools", "yes", "spec", "toolsYes"));
  } else if (caps.tools === false) {
    out.push(makeBadge("tools", "Tools", "no", "spec", "toolsNo"));
  }
  if (caps.vision_input === true) {
    out.push(makeBadge("vision", "Vision", "yes", "spec", "vision"));
  }
  if (caps.embeddings === true) {
    out.push(makeBadge("embeddings", "Embeddings", "yes", "spec", "extra"));
  }
  return out;
}

const KNOWN_METADATA_KEYS = new Set([
  "parameter_size",
  "family",
  "display_name",
  "quantization_level",
  "size",
]);

function formatMetadataKey(key: string): string {
  return key
    .replace(/_/g, " ")
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function formatMetadataValue(value: unknown): string | null {
  if (typeof value === "string" && value.trim()) return value.trim();
  if (typeof value === "number" && Number.isFinite(value)) {
    return value > 1024 ? formatBytes(value) : String(value);
  }
  if (typeof value === "boolean") return value ? "yes" : "no";
  return null;
}

function makeBadge(
  key: string,
  title: string,
  label: string,
  group: ModelDisplayBadge["group"],
  styleKey: keyof typeof MODEL_BADGE_STYLES,
): ModelDisplayBadge {
  const style = MODEL_BADGE_STYLES[styleKey];
  return {
    key,
    title,
    label,
    group,
    className: style.badge,
    valueClassName: style.value,
  };
}

/** Structured metadata for rich model selector rows. */
export function getModelDisplayParts(model: CatalogModel): ModelDisplayParts {
  const meta = model.metadata ?? {};
  const specBadges: ModelDisplayBadge[] = [];
  const statusBadges: ModelDisplayBadge[] = [];

  const parameterSize = meta.parameter_size;
  if (typeof parameterSize === "string" && parameterSize) {
    specBadges.push(
      makeBadge("params", "Params", parameterSize, "spec", "params"),
    );
  }

  const family = meta.family;
  if (typeof family === "string" && family) {
    specBadges.push(
      makeBadge("family", "Family", family, "spec", "family"),
    );
  }

  const quantization = meta.quantization_level;
  if (typeof quantization === "string" && quantization) {
    specBadges.push(
      makeBadge("quantization", "Quant", quantization, "spec", "quantization"),
    );
  }

  const size = meta.size;
  if (typeof size === "number" && size > 0) {
    specBadges.push(
      makeBadge("size", "Size", formatBytes(size), "spec", "size"),
    );
  }

  specBadges.push(...capabilityBadges(model.capabilities));

  if (model.provider && model.provider !== "unknown") {
    specBadges.push(
      makeBadge(
        "provider",
        "Provider",
        model.provider,
        "spec",
        "provider",
      ),
    );
  }

  for (const [key, value] of Object.entries(meta)) {
    if (KNOWN_METADATA_KEYS.has(key)) continue;
    const formatted = formatMetadataValue(value);
    if (!formatted) continue;
    specBadges.push(
      makeBadge(
        `extra-${key}`,
        formatMetadataKey(key),
        formatted,
        "spec",
        "extra",
      ),
    );
  }

  if (model.source === "alias") {
    statusBadges.push(
      makeBadge("alias", "Source", "alias", "status", "alias"),
    );
  } else if (model.source === "live") {
    statusBadges.push(
      makeBadge("live", "Source", "live", "status", "live"),
    );
  }

  const displayName = meta.display_name;
  const backendId =
    model.source === "alias" &&
    model.backend_id &&
    model.backend_id !== model.id
      ? model.backend_id
      : undefined;

  return {
    id: model.id,
    provider: model.provider !== "unknown" ? model.provider : undefined,
    backendId,
    displayName:
      typeof displayName === "string" &&
      displayName &&
      displayName !== model.id
        ? displayName
        : undefined,
    specBadges,
    statusBadges,
  };
}

/** Lowercase text used to match models in searchable selectors. */
export function getModelSearchText(model: CatalogModel): string {
  const parts = [
    model.id,
    model.provider,
    model.source,
    model.backend_id ?? "",
    formatModelMetadata(model),
  ];

  const meta = model.metadata ?? {};
  for (const value of Object.values(meta)) {
    if (typeof value === "string" || typeof value === "number") {
      parts.push(String(value));
    }
  }

  return parts.filter(Boolean).join(" ").toLowerCase();
}

export function filterCatalogModels(
  models: CatalogModel[],
  query: string,
): CatalogModel[] {
  const trimmed = query.trim().toLowerCase();
  if (!trimmed) return models;
  return models.filter((model) => getModelSearchText(model).includes(trimmed));
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
  providers: Array<{ id?: string; models: CatalogModel[] }>,
): CatalogModel[] {
  const byId = new Map<string, CatalogModel>();
  for (const provider of providers) {
    const providerId = provider.id ?? "";
    for (const model of withProviderId(provider.models, providerId)) {
      if (!byId.has(model.id)) {
        byId.set(model.id, model);
      }
    }
  }
  return Array.from(byId.values()).sort((a, b) => a.id.localeCompare(b.id));
}
