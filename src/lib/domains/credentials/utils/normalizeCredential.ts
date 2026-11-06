import { CredentialStatus, CredentialType, type Credential } from '../types';

type Raw = Record<string, unknown>;

function parseJson<T>(value: unknown, fallback: T): T {
  if (typeof value !== 'string') return (value as T) ?? fallback;
  try {
    return (JSON.parse(value) as T) ?? fallback;
  } catch {
    return fallback;
  }
}

function toDate(value: unknown): Date | undefined {
  if (!value) return undefined;
  // Rust sends naive UTC datetimes (no offset); without a Z, Date parses them as local time.
  const str = String(value);
  const d = new Date(/(Z|[+-]\d\d:?\d\d)$/.test(str) ? str : `${str}Z`);
  return isNaN(d.getTime()) ? undefined : d;
}

/** Map a raw Rust credential row (snake_case, JSON-string columns) to the TS `Credential`. */
export function normalizeCredential(raw: unknown): Credential {
  const r = (raw ?? {}) as Raw;
  const tags = parseJson<unknown>(r.tags, []);
  return {
    id: String(r.id ?? ''),
    name: String(r.name ?? ''),
    type: ((r.credential_type ?? r.type) as CredentialType) || CredentialType.OTHER,
    status: (r.status as CredentialStatus) || CredentialStatus.ACTIVE,
    description: (r.description as string | null) ?? undefined,
    tags: Array.isArray(tags) ? tags.map(String) : [],
    createdAt: toDate(r.created_at) ?? new Date(0),
    updatedAt: toDate(r.updated_at) ?? new Date(0),
    lastUsed: toDate(r.last_used),
    expiresAt: toDate(r.expires_at),
    encryptedValue: String(r.encrypted_value ?? ''),
    encryptedFields: parseJson(r.encrypted_fields, {}),
    metadata: parseJson(r.metadata, {}),
  };
}

export const normalizeCredentials = (raw: unknown): Credential[] =>
  Array.isArray(raw) ? raw.map(normalizeCredential) : [];
