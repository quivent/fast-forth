# Self-Update Quick Start Guide

**Status**: Implementation Checklist
**Reference**: See `SELF_MODIFYING_CLI_DESIGN.md` for full specification

---

## 1. Quick Implementation Checklist

### Phase 1: Foundation (Week 1)

```bash
# Add dependencies to Cargo.toml
[dependencies]
ed25519-dalek = "2.0"
sha2 = "0.10"
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- [ ] Add dependencies to `Cargo.toml`
- [ ] Create `backend/src/update/mod.rs`
- [ ] Create `backend/src/update/manifest.rs`
- [ ] Create `backend/src/update/signature.rs`
- [ ] Implement manifest JSON schema
- [ ] Implement Ed25519 signature verification
- [ ] Write unit tests for manifest parsing
- [ ] Write unit tests for signature verification

**Completion Criteria**: `cargo test --lib update` passes

---

### Phase 2: Core Update Logic (Week 2)

- [ ] Implement `backend/src/update/updater.rs`
- [ ] Add HTTP client for fetching manifests
- [ ] Implement file download with progress tracking
- [ ] Implement `CacheManager` in `backend/src/update/cache.rs`
- [ ] Add cache validation logic
- [ ] Implement atomic update mechanism
- [ ] Add rollback functionality
- [ ] Write integration tests

**Completion Criteria**: Can download and verify update manifest

---

### Phase 3: Cranelift Integration (Week 3)

- [ ] Modify `CraneliftBackend::compile_cached()`
- [ ] Implement `CompiledArtifact` serialization
- [ ] Add incremental compilation support
- [ ] Implement dependency tracking
- [ ] Add cache hit/miss metrics
- [ ] Benchmark compilation times (target: 10-50ms/file)
- [ ] Add cache eviction policy (LRU)

**Completion Criteria**: Incremental compilation working, benchmarks meet targets

---

### Phase 4: Hot-Swap Mechanism (Week 4)

- [ ] Implement `FunctionRegistry` in `backend/src/update/hotswap.rs`
- [ ] Implement `SafeSwapper` with grace period
- [ ] Add atomic pointer swapping
- [ ] Implement memory safety guarantees
- [ ] Test concurrent calls during swap
- [ ] Add garbage collection of old code
- [ ] Write stress tests (1000+ concurrent calls)

**Completion Criteria**: Hot-swap works without crashes or data races

---

### Phase 5: CLI Integration (Week 5)

- [ ] Add `--self-update` command to `cli/main.rs`
- [ ] Add `--rollback` command
- [ ] Implement progress indicators
- [ ] Add error recovery
- [ ] Implement automatic rollback on failure
- [ ] Add version pinning support
- [ ] Create user-facing documentation

**Completion Criteria**: End-to-end update workflow works

---

### Phase 6: Production Hardening (Week 6)

- [ ] Set up update server (S3 + CloudFront)
- [ ] Generate Ed25519 key pair (air-gapped)
- [ ] Create release signing pipeline
- [ ] Implement telemetry (opt-in)
- [ ] Add update success rate tracking
- [ ] Load test (1000+ concurrent updates)
- [ ] Security audit
- [ ] Performance profiling

**Completion Criteria**: Production deployment ready

---

## 2. Minimal Viable Implementation

For quickest path to working prototype (3-4 days):

```rust
// backend/src/update/mod.rs (minimal version)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub url: String,
    pub sha256: String,
}

pub struct Updater {
    cache_dir: PathBuf,
}

impl Updater {
    pub async fn check_for_updates(&self) -> Result<Option<Manifest>, Box<dyn std::error::Error>> {
        let url = "https://updates.fastforth.dev/manifest.json";
        let manifest: Manifest = reqwest::get(url).await?.json().await?;

        // Compare with installed version
        let installed = self.installed_version()?;
        if manifest.version != installed {
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    pub async fn install_update(&self, manifest: Manifest) -> Result<(), Box<dyn std::error::Error>> {
        for file in manifest.files {
            // Download file
            let bytes = reqwest::get(&file.url).await?.bytes().await?;

            // Verify SHA256
            let hash = sha256::digest(&bytes);
            if hash != file.sha256 {
                return Err("Hash mismatch".into());
            }

            // Save to cache
            let path = self.cache_dir.join(&file.name);
            std::fs::write(path, bytes)?;
        }

        Ok(())
    }

    fn installed_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let lock_file = self.cache_dir.join("manifest.lock");
        let content = std::fs::read_to_string(lock_file)?;
        let manifest: Manifest = serde_json::from_str(&content)?;
        Ok(manifest.version)
    }
}
```

**Day 1**: Manifest parsing + HTTP download
**Day 2**: SHA256 verification + caching
**Day 3**: CLI integration
**Day 4**: Testing + bug fixes

---

## 3. Testing Quick Reference

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let json = r#"{"version":"1.2.4","files":[]}"#;
        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.version, "1.2.4");
    }

    #[tokio::test]
    async fn test_download_file() {
        let updater = Updater::new_test();
        let file = FileEntry {
            name: "test.forth".into(),
            url: "http://localhost:8080/test.forth".into(),
            sha256: "abc123...".into(),
        };

        // Start test server
        let _server = spawn_test_server();

        let result = updater.download_file(&file).await;
        assert!(result.is_ok());
    }
}
```

### Integration Test

```rust
#[tokio::test]
async fn test_full_update_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let updater = Updater::new(temp_dir.path());

    // Install v1.2.3
    let v1_2_3 = Manifest {
        version: "1.2.3".into(),
        files: vec![/* ... */],
    };
    updater.install_update(v1_2_3).await.unwrap();
    assert_eq!(updater.installed_version().unwrap(), "1.2.3");

    // Update to v1.2.4
    let v1_2_4 = Manifest {
        version: "1.2.4".into(),
        files: vec![/* ... */],
    };
    updater.install_update(v1_2_4).await.unwrap();
    assert_eq!(updater.installed_version().unwrap(), "1.2.4");
}
```

---

## 4. Performance Targets

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Download time | < 1s | `time curl manifest.json` |
| Compile time | 5-7s | Benchmark with `criterion` |
| Startup (cached) | < 100ms | `hyperfine ./fastforth --version` |
| Runtime perf | 70-90% of C | Compare benchmarks |
| Cache hit rate | > 90% | Telemetry metrics |

---

## 5. Common Pitfalls

### 1. Signature Verification

```rust
// ❌ WRONG: Verify after compilation
compile(source)?;
verify_signature(source)?;

// ✅ CORRECT: Verify before compilation
verify_signature(source)?;
compile(source)?;
```

### 2. Atomic Updates

```rust
// ❌ WRONG: Direct write (can corrupt on crash)
fs::write(cache_file, data)?;

// ✅ CORRECT: Atomic write via temp + rename
let temp = cache_file.with_extension(".tmp");
fs::write(&temp, data)?;
fs::rename(temp, cache_file)?;  // Atomic on POSIX
```

### 3. Hot-Swap Safety

```rust
// ❌ WRONG: Immediate swap (race condition)
registry.swap(name, new_ptr);
drop(old_code);

// ✅ CORRECT: Grace period before cleanup
registry.swap(name, new_ptr);
sleep(Duration::from_secs(5));  // Let in-flight calls finish
drop(old_code);
```

---

## 6. Debugging Commands

```bash
# Check installed version
./fastforth --version

# Check for updates (dry run)
./fastforth --self-update --check-only

# View cache contents
ls -lh ~/.cache/fastforth/

# View manifest
cat ~/.cache/fastforth/manifest.lock | jq

# Clear cache (force recompile)
rm -rf ~/.cache/fastforth/compiled/

# Verify signatures manually
openssl dgst -sha256 -verify public.pem -signature file.sig file.forth

# Test update server
curl https://updates.fastforth.dev/manifest.json | jq
```

---

## 7. Next Steps After Implementation

1. **Performance Tuning**
   - Profile compilation hot paths
   - Optimize cache lookup
   - Parallelize compilation

2. **Enhanced Security**
   - Add manifest chain validation
   - Implement version pinning
   - Add rollback attack prevention

3. **Better UX**
   - Progress bars with `indicatif`
   - Changelog display
   - Interactive rollback selection

4. **Monitoring**
   - Add telemetry dashboard
   - Track update success rates
   - Monitor cache hit rates

---

## 8. Resources

- **Ed25519 Library**: https://docs.rs/ed25519-dalek/
- **Cranelift Docs**: https://docs.rs/cranelift-codegen/
- **Atomic File Writes**: https://docs.rs/tempfile/
- **HTTP Client**: https://docs.rs/reqwest/

**Reference Documentation**: `/Users/joshkornreich/Documents/Projects/FastForth/docs/SELF_MODIFYING_CLI_DESIGN.md`

---

**Last Updated**: 2025-11-14
**Status**: Ready for implementation
