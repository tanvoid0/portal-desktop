# Tauri Signing Key Management Guide

## ⚠️ Critical: Key Loss Implications

### If You Lose the Private Key File

**Consequences:**
- ❌ **Cannot sign new updates** - You won't be able to create signed update bundles
- ❌ **Cannot verify existing updates** - Users won't be able to verify update integrity
- ⚠️ **Breaking change** - You'll need to generate a NEW key pair and update the public key

**Recovery Options:**
- **None** - Cryptographic keys cannot be recovered if lost
- **Workaround**: Generate a new key pair and update `tauri.conf.json` with the new public key
- **User Impact**: Existing users will need to manually update or reinstall (their app has the old public key)

### If You Lose the Password (for password-protected keys)

**Consequences:**
- ❌ **Key is permanently unusable** - The private key cannot be decrypted without the password
- ❌ **Same as losing the key file** - You'll need to generate a new key pair

**Recovery Options:**
- **None** - Passwords cannot be recovered (by design, for security)
- **Workaround**: Generate a new key pair (same as losing the key file)

## 🔐 Best Practices

### Option 1: No Password (Recommended for CI/CD)

**Pros:**
- ✅ No password to lose
- ✅ Easier for automated builds
- ✅ Can be stored in secure CI/CD secrets

**Cons:**
- ⚠️ If key file is compromised, it's immediately usable
- ⚠️ Requires strict file system security

**Generate:**
```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/portal-desktop.key -p ''
```

**Storage:**
- Store in secure CI/CD secrets (GitHub Secrets, GitLab CI Variables, etc.)
- Use encrypted backups
- Restrict file permissions: `chmod 600 ~/.tauri/portal-desktop.key`

### Option 2: With Password (More Secure)

**Pros:**
- ✅ Extra layer of security
- ✅ Even if key file is stolen, password is needed

**Cons:**
- ⚠️ Password must be remembered/stored securely
- ⚠️ If password is lost, key is unusable
- ⚠️ Requires password input during builds

**Generate:**
```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/portal-desktop.key -p 'your-strong-password'
```

**Storage:**
- Store password in password manager (1Password, Bitwarden, etc.)
- Document password location in secure team vault
- Use environment variable: `export TAURI_KEY_PASSWORD="your-password"`

## 📦 Backup Strategy

### Essential Backups

1. **Private Key File**
   ```bash
   # Backup to secure location
   cp ~/.tauri/portal-desktop.key ~/secure-backup/portal-desktop.key.backup
   ```

2. **Public Key** (already in `tauri.conf.json`, but keep a separate copy)
   ```bash
   # Extract and save separately
   grep -A 1 "pubkey" src-tauri/tauri.conf.json > ~/secure-backup/public-key.txt
   ```

3. **Password** (if used)
   - Store in password manager
   - Document in secure team knowledge base
   - Never store in plain text files

### Backup Locations

- ✅ Encrypted cloud storage (with 2FA)
- ✅ Secure password manager
- ✅ Encrypted USB drive (stored in safe)
- ✅ Secure team vault/documentation system
- ❌ **Never**: Git repository, unencrypted cloud storage, email

## 🚨 Disaster Recovery Plan

### Scenario 1: Lost Private Key

**Steps:**
1. Generate new key pair:
   ```bash
   npx @tauri-apps/cli signer generate -w ~/.tauri/portal-desktop.key -f
   ```
   (Use `-f` flag to overwrite if file exists)

2. Update `tauri.conf.json` with new public key

3. **Important**: Update version number in `tauri.conf.json` to force re-download

4. Build and release new version with new public key

5. **User Communication**: 
   - Announce breaking change
   - Provide manual update instructions
   - Consider major version bump (e.g., 1.0.0 → 2.0.0)

### Scenario 2: Lost Password

**Steps:**
Same as Scenario 1 - you'll need to generate a new key pair.

### Scenario 3: Compromised Key

**Steps:**
1. Immediately generate new key pair
2. Update public key in config
3. Revoke/remove old signed releases if possible
4. Release new version with new key
5. Notify users of security update

## 🔄 Key Rotation Strategy

### When to Rotate

- ✅ Annually (security best practice)
- ✅ After team member with access leaves
- ✅ After suspected compromise
- ✅ Before major version releases

### Rotation Process

1. Generate new key pair
2. Update `tauri.conf.json` with new public key
3. Bump version number
4. Build and release with new key
5. Archive old key securely (don't delete immediately)

## 📝 Checklist for Production

- [ ] Generate signing key pair
- [ ] Store private key securely (password manager/secure vault)
- [ ] Store password (if used) in password manager
- [ ] Create encrypted backups of private key
- [ ] Update `tauri.conf.json` with public key
- [ ] Test signing process in staging
- [ ] Document key location for team
- [ ] Set up CI/CD with key in secrets
- [ ] Verify `.gitignore` excludes private keys
- [ ] Test update process end-to-end

## 💡 Recommendations

**For Solo Developer:**
- Use password-protected key
- Store password in password manager
- Create encrypted backup on external drive

**For Team:**
- Use password-less key (easier for CI/CD)
- Store in secure CI/CD secrets
- Document in team knowledge base
- Use key rotation schedule

**For Enterprise:**
- Use hardware security module (HSM) if available
- Implement key rotation policy
- Use separate keys for dev/staging/production
- Maintain key escrow/backup procedures

