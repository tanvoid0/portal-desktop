/**
 * Credentials Domain Types
 */

export enum CredentialType {
	SSH_KEY = 'ssh_key',
	API_TOKEN = 'api_token',
	ENV_VAR = 'env_var',
	DATABASE = 'database',
	CLOUD_PROVIDER = 'cloud_provider',
	REGISTRY = 'registry',
	OTHER = 'other'
}

export enum CredentialStatus {
	ACTIVE = 'active',
	INACTIVE = 'inactive',
	EXPIRED = 'expired',
	REVOKED = 'revoked'
}

export interface Credential {
	id: string;
	name: string;
	type: CredentialType;
	status: CredentialStatus;
	description?: string;
	tags: string[];
	createdAt: Date;
	updatedAt: Date;
	lastUsed?: Date;
	expiresAt?: Date;
	// Encrypted fields (never store plain text)
	encryptedValue: string;
	encryptedFields: Record<string, string>;
	// Metadata
	metadata: CredentialMetadata;
}

export interface CredentialMetadata {
	provider?: string;
	environment?: string;
	project?: string;
	username?: string;
	host?: string;
	port?: number;
	database?: string;
	table?: string;
	keyId?: string;
	fingerprint?: string;
	algorithm?: string;
	keySize?: number;
	format?: string;
	notes?: string;
}

export interface SecureVault {
	id: string;
	name: string;
	description?: string;
	credentials: Credential[];
	createdAt: Date;
	updatedAt: Date;
	// Encryption settings
	encryptionKey: string; // Master key for this vault
	keyDerivation: KeyDerivationSettings;
}

export interface KeyDerivationSettings {
	algorithm: string; // e.g., 'pbkdf2', 'scrypt', 'argon2'
	iterations: number;
	salt: string;
	keyLength: number;
}

export interface CredentialCreateRequest {
	name: string;
	type: CredentialType;
	description?: string;
	tags?: string[];
	value: string; // Will be encrypted
	fields?: Record<string, string>; // Additional encrypted fields
	metadata?: CredentialMetadata;
	expiresAt?: Date;
}

export interface CredentialUpdateRequest {
	name?: string;
	description?: string;
	tags?: string[];
	value?: string;
	fields?: Record<string, string>;
	metadata?: CredentialMetadata;
	status?: CredentialStatus;
	expiresAt?: Date;
}

export interface CredentialDecryptRequest {
	credentialId: string;
	masterPassword?: string;
	sessionToken?: string;
}

export interface CredentialDecryptResult {
	success: boolean;
	value?: string;
	fields?: Record<string, string>;
	error?: string;
}

export interface SSHKeyCredential extends Credential {
	type: CredentialType.SSH_KEY;
	metadata: SSHKeyMetadata;
}

export interface SSHKeyMetadata extends CredentialMetadata {
	publicKey: string;
	privateKey: string; // Encrypted
	keyType: 'rsa' | 'ed25519' | 'ecdsa' | 'dsa';
	keySize: number;
	fingerprint: string;
	comment?: string;
}

export interface APITokenCredential extends Credential {
	type: CredentialType.API_TOKEN;
	metadata: APITokenMetadata;
}

export interface APITokenMetadata extends CredentialMetadata {
	token: string; // Encrypted
	scopes: string[];
	permissions: string[];
	rateLimit?: {
		requests: number;
		period: string;
	};
}

export interface DatabaseCredential extends Credential {
	type: CredentialType.DATABASE;
	metadata: DatabaseMetadata;
}

export interface DatabaseMetadata extends CredentialMetadata {
	host: string;
	port: number;
	database: string;
	username: string;
	password: string; // Encrypted
	connectionString?: string; // Encrypted
	ssl?: boolean;
	timeout?: number;
}

export interface CloudProviderCredential extends Credential {
	type: CredentialType.CLOUD_PROVIDER;
	metadata: CloudProviderMetadata;
}

export interface CloudProviderMetadata extends CredentialMetadata {
	provider: 'aws' | 'gcp' | 'azure' | 'digitalocean' | 'linode' | 'vultr';
	accessKey: string; // Encrypted
	secretKey: string; // Encrypted
	region?: string;
	accountId?: string;
	projectId?: string;
}

export interface CredentialSearchRequest {
	query?: string;
	type?: CredentialType;
	status?: CredentialStatus;
	tags?: string[];
	provider?: string;
	environment?: string;
	project?: string;
	limit?: number;
	offset?: number;
}

export interface CredentialSearchResult {
	credentials: Credential[];
	total: number;
	page: number;
	limit: number;
}

export interface VaultCreateRequest {
	name: string;
	description?: string;
	masterPassword: string;
	keyDerivation?: Partial<KeyDerivationSettings>;
}

export interface VaultUnlockRequest {
	vaultId: string;
	masterPassword: string;
	sessionDuration?: number; // minutes
}

export interface VaultUnlockResult {
	success: boolean;
	sessionToken?: string;
	expiresAt?: Date;
	error?: string;
}

export interface EncryptionResult {
	encrypted: string;
	iv: string;
	tag: string;
	algorithm: string;
}

export interface DecryptionRequest {
	encrypted: string;
	iv: string;
	tag: string;
	algorithm: string;
	key: string;
}

export interface DecryptionResult {
	success: boolean;
	decrypted?: string;
	error?: string;
}
