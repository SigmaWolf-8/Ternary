# QC-R1 Agent 3: PlenumNET Integration Specialist Review

**Document Under Review:** `services/inter-cube/deploy-yoda.ps1` (v0.5.0, 1652 lines)
**Review Protocol Version:** 1.1.2
**Date:** 2026-03-30

---

### Finding 1
- **Section:** Lines 503-526, Binary integrity — SHA-256 as primary/fallback hash
- **Severity:** CRITICAL
- **Finding:** The deployer computes binary integrity using `Get-FileHash -Algorithm SHA256` (lines 504, 529) as the primary mechanism. TIS-27 is attempted via the daemon's `CUBE_MODE=hash` but is treated as optional. The fallback (line 524) prefixes the SHA-256 hash with `sha256:` and uses it as the integrity value. **SHA-256 is a banned primitive** in the PlenumNET framework. TIS-27 is the sole hash/MAC primitive — BLAKE3 and SHA-256 have been explicitly removed. Using SHA-256 for the most critical integrity check in the deployment pipeline (verifying the daemon binary was not tampered with) contradicts the framework's zero-external-crypto requirement. The `sha256:` prefix tag does not make this acceptable — it normalizes the use of a banned primitive.
- **Recommendation:** (1) Remove all `Get-FileHash -Algorithm SHA256` calls. (2) Make TIS-27 hashing mandatory. If the daemon cannot produce a TIS-27 hash (e.g., on first build), the deployer must fail with a clear error explaining the bootstrap gap. (3) Consider a two-phase approach: build the binary, then use the built binary to hash itself via `CUBE_MODE=hash`, and treat that as the sole integrity value.
- **Verification:** Grep the script for `SHA256` — zero occurrences should remain. Deploy and confirm the binary hash in the deployment payload is a TIS-27 hash, not a SHA-256 hash.
- **Crypto Status:** INCORRECT — SHA-256 is explicitly banned; TIS-27 is the sole permitted hash primitive.
- **Finding ID:** R1-A3-1

### Finding 2
- **Section:** Lines 154-174, `Get-TlDsaSignature` — primitive naming and context binding
- **Severity:** IMPORTANT
- **Finding:** The function is named `Get-TlDsaSignature` but the script header (line 5) and identity generation output refer to "PT26-DSA" identities. TL-DSA and PT26-DSA are the same primitive (PT26-DSA is the parameterized name, TL-DSA is the framework-level name), but the inconsistent naming in the deployer could cause confusion. More critically, the signing payload construction (line 1388) uses `"CRS-REGISTER||$publicKey||$endpoint||$timestamp"` — the `$endpoint` contains an IP:port string (e.g., `192.168.1.5:11124`), not a Rep C address. Per INVARIANT 9, all cryptographic operations that bind node identity must use Rep C addressing exclusively. The endpoint IP address is not a Rep C identifier, and binding it into the TL-DSA signature context violates the invariant.
- **Recommendation:** (1) Replace `$endpoint` in the signing payload with the node's Rep C address. The payload should be `"CRS-REGISTER||$publicKey||$repCAddress||$timestamp"`. (2) If the Rep C address is not yet assigned at registration time (chicken-and-egg problem), use the node's public key hash or a deterministic Rep C derivation. (3) Standardize naming: use "TL-DSA" consistently in the deployer script, with a comment noting the PT26-DSA parameterization.
- **Verification:** Inspect the CRS registration payload and confirm the signed context contains only Rep C addresses, public keys, and timestamps — no IP addresses, hostnames, or port numbers.
- **Crypto Status:** INCORRECT — signature context binds IP:port (non-Rep-C identifier) violating INVARIANT 9.

### Finding 3
- **Section:** Lines 1440-1445, Deployment signature — payload structure
- **Severity:** IMPORTANT
- **Finding:** The deployment signature payload (line 1441) is constructed as `"DEPLOYMENT||$addresses||$timestamp"` where `$addresses` are the registered Rep C addresses joined by `||` and `$timestamp` is an ISO 8601 string. The context string `"DEPLOYMENT"` is not in the canonical context string registry documented in `plenumnet-repo-guide/SKILL.md`. Context strings are load-bearing — using an unregistered context string means this signature domain is not formally defined and could collide with future context strings. Additionally, the timestamp uses ISO 8601 format rather than the Salvi epoch femtosecond timestamp (INVARIANT 6), which creates a domain inconsistency between the deployer and the runtime system.
- **Recommendation:** (1) Register `"DEPLOYMENT"` in the canonical context string registry, or use an existing registered context string if one applies. (2) Consider using Salvi epoch timestamps for consistency with the runtime, or document why ISO 8601 is appropriate at deploy time (e.g., Salvi epoch infrastructure not yet available). (3) Define the exact payload format in a versioned specification document.
- **Verification:** Check the canonical context string registry for `"DEPLOYMENT"`. If absent, file a request to add it. Verify the signature can be verified by the remote CRS using the same context string.
- **Crypto Status:** UNVERIFIED — context string `"DEPLOYMENT"` not found in canonical registry.

### Finding 4
- **Section:** Lines 1386-1396, CRS registration — signing context
- **Severity:** IMPORTANT
- **Finding:** The CRS registration signing context (line 1388) is `"CRS-REGISTER||$publicKey||$endpoint||$timestamp"`. The context string `"CRS-REGISTER"` is not in the canonical context string registry. Additionally, the `||` delimiter is ad-hoc — the framework's context string convention uses specific formatting (typically hyphenated identifiers like `"PlenumNET-CON-v2.5"`, `"HEARTBEAT-MAC"`). An ad-hoc delimiter creates ambiguity: if any field contains `||`, the context becomes unparseable. The payload structure should follow the established PlenumNET context string conventions.
- **Recommendation:** (1) Register `"CRS-REGISTER"` in the canonical context string registry with a formal definition of its payload structure. (2) Use a structured serialization format (e.g., length-prefixed fields or a canonical JSON representation) rather than `||` delimiters. (3) Align with existing context string patterns in the codebase.
- **Verification:** Search the codebase for `"CRS-REGISTER"` context string usage and confirm it matches the deployer's payload format. If not found, file a registration request.
- **Crypto Status:** UNVERIFIED — context string `"CRS-REGISTER"` not found in canonical registry.

### Finding 5
- **Section:** Lines 22-45, Array3 topology — port range and node ID conventions
- **Severity:** MINOR
- **Finding:** The port allocation and node ID conventions are correctly documented and implemented. Node IDs use Rep C ordinals {1, 2, 3} (line 44: "Node IDs are Rep C ordinals {1,2,3} — NOT GF(3) {0,1,2}. Zero is never used as a node ID."). The 27-slot cube topology (SLOTS_PER_NODE = 27 = 3^3) and gateway offset 13 are geometrically correct per INVARIANT 4 (13 = T₇ = 1 ternary radian). The gateway formula `BASE_PORT + ((CUBE_NODE_ID - 1) * 27) + 13` correctly places the gateway at the center slot [2,2,2] of each node's 27-slot cube. **However**, the comment says "GATEWAY_OFFSET = 13" but the center of a 27-element cube (indexed 0-26) is at position 13, which corresponds to the 3D coordinate [1,1,1] in 0-indexed terms or [2,2,2] in 1-indexed Rep C terms. The comment should clarify which indexing is used.
- **Recommendation:** Add a clarifying comment: "GATEWAY_OFFSET = 13 corresponds to the 14th slot (0-indexed position 13), which is the center of the 3×3×3 cube at coordinate [2,2,2] in Rep C (1-indexed) or [1,1,1] in GF(3) (0-indexed)."
- **Verification:** Verify that `13 = (1*9 + 1*3 + 1)` in GF(3) coordinates, confirming the center slot.
- **Crypto Status:** N/A

### Finding 6
- **Section:** Lines 734-758, Identity generation — keygen invocation
- **Severity:** IMPORTANT
- **Finding:** The keygen invocation sets `CUBE_MODE=keygen` and `CUBE_IDENTITY_DIR` to generate PT26-DSA key pairs. The script then extracts the public key by running the daemon again with `CUBE_MODE=keygen` (line 751) and parsing the output for "PT26-DSA Public Key" or "pk:" lines. This means keygen is invoked twice for each node — once to generate and once to read. The second invocation might regenerate keys if the daemon's keygen mode does not distinguish between "generate new" and "read existing". This could silently overwrite the keys generated in the first invocation. The script relies on the daemon being idempotent in keygen mode (only generating if `master.key` doesn't exist), but this assumption is not verified or documented.
- **Recommendation:** (1) Use a separate `CUBE_MODE=info` or `CUBE_MODE=pubkey` to read existing key material without risk of regeneration. (2) If keygen mode is used for both operations, document the idempotency contract explicitly. (3) Verify that the public key read after the second invocation matches by comparing with a direct file read of the public key component.
- **Verification:** Run keygen twice on the same identity directory and confirm the `master.key` file is byte-identical after both invocations.
- **Crypto Status:** UNVERIFIED — keygen idempotency contract is assumed but not documented.

### Finding 7
- **Section:** Lines 698-719, Identity migration — SHA-256 for integrity verification
- **Severity:** IMPORTANT
- **Finding:** The identity migration from the old location (`$env:USERPROFILE\.plenumnet`) to the new location (`C:\PlenumNET\plenumnet-data`) uses SHA-256 (lines 708-709) to verify the integrity of the migrated `master.key` file: `Get-FileHash -Algorithm SHA256`. This is another instance of the banned SHA-256 primitive being used for a security-critical integrity check — verifying that cryptographic key material was not corrupted during migration. Cross-reference: Agent 1 (Security Engineer) should assess the cryptographic implications.
- **Recommendation:** (1) Replace SHA-256 with TIS-27 for migration integrity verification. (2) If TIS-27 is not available at this point in the deployment (the binary may not be built yet for a fresh install), document this as a bootstrap gap. For upgrades, the binary is already available and TIS-27 should be used.
- **Verification:** Grep the script for `SHA256` after the fix — zero occurrences should remain.
- **Crypto Status:** INCORRECT — SHA-256 is banned; TIS-27 must be used for all integrity checks.

### Finding 8
- **Section:** Lines 760-774, Daemon configuration — key lifecycle boundary
- **Severity:** MINOR
- **Finding:** The deployer generates initial key material (`master.key`) and passes configuration to services via environment variables. The script does not document the key lifecycle boundary between the deployer (installer) and the runtime daemon. Specifically: (1) Does the deployer generate the initial key epoch? (2) Is the generated key material valid for the first 14-day rotation period (per `ARC_EPOCH_SECS / RADIAN_DEG = 1,209,600 seconds`)? (3) What happens if the services are not started within the first rotation period? The key rotation logic is entirely delegated to the daemon, which is correct, but the deployer should document what it provisions and what the daemon manages.
- **Recommendation:** (1) Add a comment block documenting: "The deployer provisions the initial PT26-DSA keypair. Key rotation is managed by the daemon runtime at 14-day intervals (1,209,600 seconds). The initial key has no expiry — the daemon begins rotation from the first epoch after startup." (2) Verify this matches the daemon's actual behavior.
- **Verification:** Deploy, wait 15 days without starting services, then start them. Confirm the initial key is still valid and rotation begins correctly.
- **Crypto Status:** UNVERIFIED — key lifecycle boundary not explicitly documented.

### Finding 9
- **Section:** Lines 1447-1469, Deployment payload — metadata field
- **Severity:** IMPORTANT
- **Finding:** The deployment payload sent to the remote CRS includes a `metadata` object containing `hostname`, `ip`, `architecture`, `localCrsUrl`, `binaryPath`, `binarySizeMB`, `logDir`, and `identityBase`. Per INVARIANT 9, nodes must be identified by Rep C addresses in all contexts. The `hostname` and `ip` fields are non-Rep-C identifiers. The `binaryPath`, `logDir`, and `identityBase` fields leak filesystem layout information to a remote endpoint. While this metadata may be useful for operational dashboards, it violates the principle that Rep C addresses are the sole identity mechanism. The `localCrsUrl` contains `localhost:11124` which exposes the port allocation strategy.
- **Recommendation:** (1) Remove `hostname`, `ip`, `binaryPath`, `logDir`, `identityBase`, and `localCrsUrl` from the remote deployment payload. (2) If operational metadata is needed, create a separate opt-in telemetry endpoint with explicit user consent. (3) The deployment payload should contain only: `addresses` (Rep C), `daemonCount`, `publicKeys`, `releaseTag`, `timestamp`, `signature`, and `binaryHash` (TIS-27).
- **Verification:** Capture the deployment payload and verify it contains only Rep C addresses and public keys as node identifiers.
- **Crypto Status:** N/A (identity/addressing concern, not crypto primitive)

### Finding 10
- **Section:** Lines 807-813 and 839-847, Service wrapper — RELAY_URL configuration
- **Severity:** MINOR
- **Finding:** The service wrapper `.bat` files set `RELAY_URL=$REMOTE_CRS` which resolves to `https://plenumnet.replit.app`. This is an HTTPS URL (not a TDNS address). At install time, the TDNS resolver is not available (it requires the daemon to be running), so using an HTTPS URL is correct for bootstrap. However, there is no mechanism to update `RELAY_URL` to a TDNS-resolved address after the daemon is operational. The wrapper files are static — they must be regenerated to change the relay URL.
- **Recommendation:** (1) Document that `RELAY_URL` uses HTTPS during bootstrap because the TDNS resolver is not yet available. (2) Consider having the daemon resolve the TDNS address at runtime and override the bootstrap URL, rather than relying on the static wrapper configuration. (3) Add a mechanism to update wrapper files without full re-deployment.
- **Verification:** Confirm that the daemon can resolve the relay endpoint via TDNS at runtime and does not permanently depend on the bootstrap HTTPS URL.
- **Crypto Status:** N/A

---

## Summary Verdict

**Verdict: FAIL**

The deployer contains one CRITICAL finding that blocks implementation:

1. **R1-A3-1**: SHA-256 is used as the primary and fallback binary integrity hash. SHA-256 is a banned primitive in the PlenumNET framework — TIS-27 is the sole permitted hash/MAC primitive. This is not a minor naming issue; it means the most security-critical integrity check in the deployment pipeline uses a primitive the framework has explicitly removed.

Additionally, five IMPORTANT findings require resolution before first release:

2. **R1-A3-2**: TL-DSA signing context binds IP:port endpoints (non-Rep-C identifiers), violating INVARIANT 9.
3. **R1-A3-3**: The `"DEPLOYMENT"` context string is not in the canonical registry — unverified domain separation.
4. **R1-A3-4**: The `"CRS-REGISTER"` context string is not in the canonical registry — unverified domain separation.
5. **R1-A3-6**: Keygen idempotency is assumed but not verified — double invocation could overwrite keys.
6. **R1-A3-7**: Identity migration uses SHA-256 for integrity verification — another banned primitive usage.
7. **R1-A3-9**: Deployment payload transmits non-Rep-C identifiers (hostname, IP, filesystem paths) to remote endpoint.

Three MINOR findings address documentation gaps (gateway offset indexing, key lifecycle boundary, relay URL bootstrap).

**Cryptographic Status Summary:**

| Claim | Status |
|-------|--------|
| SHA-256 binary integrity hash | INCORRECT — banned primitive |
| SHA-256 migration integrity check | INCORRECT — banned primitive |
| TL-DSA signing (Get-TlDsaSignature) | INCORRECT — context binds IP:port, not Rep C |
| PT26-DSA keygen (CUBE_MODE=keygen) | UNVERIFIED — idempotency not documented |
| "CRS-REGISTER" context string | UNVERIFIED — not in canonical registry |
| "DEPLOYMENT" context string | UNVERIFIED — not in canonical registry |
| Node IDs Rep C {1,2,3} | VERIFIED — correctly implemented |
| 27-slot cube topology | VERIFIED — geometrically correct |
| Gateway offset 13 = T₇ | VERIFIED — matches INVARIANT 4 |
| Key rotation 14-day period | UNVERIFIED — lifecycle boundary not documented |
