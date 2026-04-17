# ES256 (ECDSA P-256) JWT Support Research

## TASK-SEC-LOW-01-C — Research ES256 Compatibility with Bitwarden Clients

**Status**: Research Complete — Implementation NOT recommended at this time  
**Date**: 2026-04-15  
**Researcher**: Security Sprint 4

---

## Findings

### Current State

Vaultwarden currently uses **RS256** (RSA-SHA256) for JWT signing, using a 2048-bit RSA key pair.

The JWT header is:
```json
{"alg": "RS256", "typ": "JWT"}
```

### Bitwarden Client JWT Algorithm Expectations

Bitwarden's official clients (Web, Desktop, Mobile, CLI) validate JWTs issued by the identity server. Analysis of the Bitwarden open-source clients and server SDK:

| Client | Algorithm Verification | Notes |
|---|---|---|
| Web Vault (`jslib`) | **Strict RS256 expected** | Uses `jwt-decode` which does NOT verify signature — algorithm mismatch does NOT cause client-side failure |
| Mobile (C# MAUI) | **No explicit alg check** in client decode | Server-side MUST match |
| Desktop (Electron) | Same as Web Vault | Signature not verified client-side |
| CLI (`bw`) | No signature verification | JWT decoded but alg ignored |
| Push relay (Bitwarden.com) | Uses its own tokens — unrelated | N/A |

**Key finding**: Bitwarden clients do NOT perform cryptographic signature verification on the JWT — they only decode the payload. This means clients themselves would not break if the algorithm changed.

However, the **official Bitwarden Identity server** always issues RS256 tokens. Any third-party service or Bitwarden feature that validates the JWT (e.g., emergency access, organization auth audits) expects RS256. Additionally, future client versions may add stricter algorithm pinning.

### ES256 Advantages

| Property | RS256 (current) | ES256 (ECDSA P-256) |
|---|---|---|
| Key size | 2048-bit RSA | 256-bit EC |
| JWT size | ~350 bytes signature | ~86 bytes signature |
| Sign speed | Slow (RSA | Fast (ECDSA) |
| Verify speed | Fast (RSA) | Fast (ECDSA) |
| NIST post-quantum | Not recommended post-2030 | Also not quantum-safe, but smaller and faster |

### Implementation Effort

To support ES256 in Vaultwarden:
1. Replace `openssl::rsa::Rsa` with `openssl::ec::EcKey` (P-256 curve)
2. Replace `EncodingKey::from_rsa_pem` with `EncodingKey::from_ec_pem`
3. Update `JWT_HEADER` algorithm from `jsonwebtoken::Algorithm::RS256` to `Algorithm::ES256`
4. Update key rotation: archive + generate EC key instead of RSA
5. Update `JWT_ALGORITHM` constant used in all `Validation` structs

### Risks

1. **Protocol compatibility**: The Bitwarden protocol spec is not formally documented for algorithm negotiation. Changing the algorithm may break with:
   - Bitwarden mobile apps that do signature validation (future versions)
   - SAML/SSO integrations that re-validate the JWT
   - Push notification relay HMAC signature (unrelated — uses separate key)

2. **Migration**: Existing tokens (up to 90-day refresh) signed with RS256 would fail to validate against ES256 public key — full session invalidation required.

3. **Not quantum-safe**: ES256 uses elliptic curves which are also broken by sufficiently powerful quantum computers. The only true quantum-safe option would be lattice-based algorithms (e.g., ML-DSA/CRYSTALS-Dilithium), which are not yet supported by `jsonwebtoken` crate.

---

## Recommendation

**Do NOT implement ES256 at this time.**

Rationale:
- Bitwarden clients do not verify JWT signatures — the performance gain is irrelevant
- Algorithm change risks silent breakage with future client updates
- ES256 is not post-quantum safe (same weakness as RSA for quantum attackers)
- The RSA key rotation infrastructure (LOW-01-A/B) is more valuable for security posture

**Revisit when:**
- `jsonwebtoken` crate adds ML-DSA/CRYSTALS-Dilithium support  
- Bitwarden clients formally document their algorithm negotiation behavior
- NIST post-quantum migration timeline becomes critical (NIST target: 2030–2035)

---

## References

- [jsonwebtoken crate supported algorithms](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/enum.Algorithm.html)
- [NIST IR 8547: Transition to Post-Quantum Standards](https://csrc.nist.gov/pubs/ir/8547/ipd)
- [Bitwarden client jslib JWT decode](https://github.com/bitwarden/clients/blob/main/libs/common/src/auth/services/token.service.ts)
- [RFC 7518 — JSON Web Algorithms (JWA)](https://www.rfc-editor.org/rfc/rfc7518)
