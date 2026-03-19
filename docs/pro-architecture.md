# Sentrux Pro Architecture

## Goal

Split sentrux into a fully open, trustable free binary and a separately downloadable Pro plugin. Free users get a binary with zero private code. Pro users get additional features via a runtime-loaded dylib.

## Principles

1. **Free binary = 100% public source.** Anyone can `cargo build` and verify.
2. **Pro features live in pro.dylib**, not behind `if tier.is_pro()` flags in the free binary.
3. **License key = Ed25519 signed JSON.** Offline validation, no server needed.
4. **Per-user watermarked dylib.** Each download contains the buyer's identity.
5. **Source-available Pro code.** BSL license — users can read and audit, not redistribute.

## User Flow

```
$ brew install sentrux/tap/sentrux     # free binary, public repo, zero private code

$ sentrux                               # fully functional: 5 root causes, treemap, MCP
                                        # 3 color modes, rules engine, session diff

$ sentrux login                         # opens browser
  → browser: sign in with GitHub
  → browser: purchase Pro ($15/month via Stripe/LemonSqueezy)
  → browser: "Success! Copy this activation key."

$ sentrux pro activate <key>            # saves license, downloads per-user watermarked pro.dylib
  → License verified.
  → Downloading Pro plugin for darwin-arm64...
  → Saved to ~/.sentrux/pro/
  → Restart sentrux to enable Pro features.

$ sentrux                               # Pro features active
```

## File Layout

```
~/.sentrux/
├── plugins/                # tree-sitter grammars (existing)
│   ├── rust/
│   ├── python/
│   └── ...
├── pro/                    # Pro plugin (NEW)
│   ├── pro.dylib           # per-user watermarked binary
│   └── manifest.json       # version, checksum, platform
├── license.key             # Ed25519 signed license (NEW)
├── last_update_check       # telemetry timestamp (existing)
└── telemetry_pending.json  # usage counters (existing)
```

## License Key Format

```json
{
  "user": "github:yjing",
  "email": "yjing@sentrux.dev",
  "tier": "pro",
  "issued": "2026-03-18",
  "expires": "2026-04-18",
  "id": "lic_a1b2c3d4e5f6",
  "sig": "base64_ed25519_signature_of_all_fields_above"
}
```

Validation:
1. Parse JSON
2. Verify Ed25519 signature against hardcoded public key
3. Check `expires >= today`
4. If valid → set `Tier::Pro`

No server call. No internet. Pure math.

## Pro Plugin Architecture

### What the free binary does NOT contain

Pro computation code is IN the dylib, not gated by `tier.is_pro()` in the free binary:

| Feature | Where the code lives |
|---------|---------------------|
| C4 diagram generation | pro.dylib |
| What-if simulation | pro.dylib |
| Advanced diagnostics (god files, dead code lists) | pro.dylib |
| Extra color modes (Age, Churn, Risk, Git, ExecDepth, BlastRadius) | pro.dylib |
| File detail panel (per-function metrics) | pro.dylib |
| Evolution details (hotspots, coupling pairs, bus factor) | pro.dylib |
| Context window optimizer | pro.dylib |
| Refactoring planner | pro.dylib |

### What the free binary DOES contain

| Feature | Always available |
|---------|-----------------|
| Scanning (52 languages, tree-sitter) | ✅ |
| Quality signal (5 root causes, 0-10000) | ✅ |
| Treemap visualization | ✅ |
| 3 color modes (Monochrome, Language, Heat) | ✅ |
| MCP server (scan, health scores, session diff) | ✅ |
| Rules engine (sentrux check) | ✅ |
| Quality gate (sentrux gate) | ✅ |

### Plugin loading mechanism

```rust
// In free binary at startup:
fn try_load_pro() {
    // 1. Read license
    let key_path = home_dir().join(".sentrux/license.key");
    let key_json = fs::read_to_string(&key_path).ok()?;

    // 2. Validate (offline, Ed25519)
    let license = validate_license(&key_json)?;
    if license.expired() { return; }

    // 3. Verify watermark matches license
    let dylib_path = home_dir().join(".sentrux/pro/pro.dylib");
    let watermark = read_watermark(&dylib_path)?;
    if watermark.user_id != license.id { return; }  // mismatch = stolen dylib

    // 4. Load plugin
    let lib = libloading::Library::new(&dylib_path)?;
    let init: Symbol<fn(&ProContext)> = lib.get(b"sentrux_pro_init")?;

    // 5. Pass context — Pro plugin registers its features
    init(&ProContext {
        tier: license.tier,
        register_color_mode: ...,
        register_mcp_tool: ...,
        register_panel: ...,
    });

    license::set_tier(license.tier);
}
```

### Pro plugin interface (cdylib exports)

```rust
// sentrux-pro/src/lib.rs — compiled to pro.dylib
#[no_mangle]
pub extern "C" fn sentrux_pro_init(ctx: &ProContext) {
    // Register Pro color modes
    ctx.register_color_mode("age", color_by_age);
    ctx.register_color_mode("churn", color_by_churn);
    ctx.register_color_mode("risk", color_by_risk);
    ctx.register_color_mode("git", color_by_git);
    ctx.register_color_mode("exec_depth", color_by_exec_depth);
    ctx.register_color_mode("blast_radius", color_by_blast_radius);

    // Register Pro MCP tools
    ctx.register_mcp_tool("diagnostics", handle_diagnostics);
    ctx.register_mcp_tool("whatif", handle_whatif);
    ctx.register_mcp_tool("c4", handle_c4);

    // Register Pro panels
    ctx.register_panel("file_detail", draw_file_detail);
    ctx.register_panel("evolution", draw_evolution_full);
    ctx.register_panel("whatif", draw_whatif);
}
```

## Per-User Watermark

```
Base pro.dylib has:
  static WATERMARK: [u8; 64] = [0x00; 64];

CDN at download:
  1. Read base pro.dylib for user's platform
  2. Find the 64-byte zero block
  3. Replace with: license_id (32 bytes) + HMAC(license_id, server_secret) (32 bytes)
  4. Serve to user

Runtime:
  1. Read WATERMARK from own binary
  2. Compare license_id with license.key
  3. Mismatch → refuse to load (dylib belongs to someone else)
```

## Anti-Piracy Posture

**Defense in depth, not bulletproof:**

| Defense | What it stops |
|---------|--------------|
| Pro code IN dylib, not in free binary | Fake dylib can't compute real features |
| License key Ed25519 validation | Can't forge keys without private key |
| Per-user watermark | Leaked dylib traceable to source |
| Watermark ↔ license cross-check | Can't mix someone else's dylib with your key |
| Telemetry license_id + ip_hash | Detect shared keys across many IPs |

**What we accept:**
- Binary patching will always be possible. We don't fight it.
- The tiny number of people who crack a $15/month dev tool weren't going to pay.
- Engineering time on DRM > revenue lost to piracy.

## Build Pipeline

```
Public repo (sentrux/sentrux):
  CI builds → sentrux-darwin-arm64 (free binary)
  Homebrew installs this.

Private repo (sentrux/sentrux-pro):
  CI builds → pro-darwin-arm64.dylib (base, no watermark)
  Uploaded to CDN (private, not public GitHub release)

CDN Worker (api.sentrux.dev):
  On download request with valid license:
    1. Fetch base dylib from R2/S3
    2. Embed watermark
    3. Serve to user
```

## CLI Commands

```bash
sentrux login                  # opens browser → GitHub OAuth → purchase
sentrux pro activate <key>     # saves license, downloads watermarked dylib
sentrux pro status             # shows tier, expiry, features loaded
sentrux pro update             # re-downloads latest dylib (re-watermarked)
sentrux pro deactivate         # removes license + dylib, back to free
```

## Implementation Order

1. **License key validation** — Ed25519 verify in sentrux-core (add `ed25519-dalek` crate)
2. **try_load_pro()** — runtime dylib loading with license + watermark check
3. **ProContext interface** — registration API for color modes, MCP tools, panels
4. **Move Pro features OUT of free binary** — into sentrux-pro cdylib
5. **sentrux login/pro CLI commands** — in sentrux-bin
6. **CDN watermark worker** — Cloudflare Worker on api.sentrux.dev
7. **Website purchase flow** — Stripe/LemonSqueezy integration
8. **Free binary CI** — release.yml builds from public repo only

## Revenue Model

| Tier | Price | Features |
|------|-------|----------|
| Free | $0 | Full scanning, scoring, treemap, 3 colors, MCP (scores only), rules |
| Pro | $15/month | + 6 colors, diagnostics, what-if, C4, evolution, file detail, context optimizer |
| Team | $40/month/seat | + shared dashboard, architecture drift alerts, PR annotations, SSO |
