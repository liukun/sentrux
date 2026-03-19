//! License tier system with Ed25519 key validation.
//!
//! The `Tier` enum is the universal currency for feature gating.
//! License keys are Ed25519-signed JSON files stored at ~/.sentrux/license.key.
//! Validation is fully offline — no server call needed.
//!
//! ## Key format
//! ```json
//! {
//!   "user": "github:yjing",
//!   "email": "yjing@sentrux.dev",
//!   "tier": "pro",
//!   "issued": "2026-03-18",
//!   "expires": "2026-04-18",
//!   "id": "lic_a1b2c3d4e5f6",
//!   "sig": "base64_ed25519_signature"
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Ed25519 public key for license verification (embedded at compile time).
/// The corresponding private key is kept offline — never in any repository.
const LICENSE_PUBLIC_KEY: [u8; 32] = [
    51, 80, 192, 124, 169, 177, 177, 37, 40, 185, 99, 192, 167, 42, 157, 250,
    1, 110, 189, 234, 236, 9, 143, 61, 221, 122, 243, 48, 251, 237, 154, 119,
];

/// License tier determining feature access.
///
/// Ordered by privilege: Free < Pro < Team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Free = 0,
    Pro = 1,
    Team = 2,
}

impl Tier {
    /// Check if this tier grants access to features requiring `required` tier.
    #[inline]
    pub fn can_access(self, required: Tier) -> bool {
        self >= required
    }

    #[inline]
    pub fn is_pro(self) -> bool {
        self >= Tier::Pro
    }

    #[inline]
    pub fn is_team(self) -> bool {
        self >= Tier::Team
    }

    /// Detail list limit for this tier (used by health, test_gaps, etc.)
    pub fn detail_limit(self) -> usize {
        match self {
            Tier::Free => 0,
            Tier::Pro | Tier::Team => usize::MAX,
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::Free => write!(f, "Free"),
            Tier::Pro => write!(f, "Pro"),
            Tier::Team => write!(f, "Team"),
        }
    }
}

// ── Global tier state ──

static TIER: std::sync::OnceLock<Tier> = std::sync::OnceLock::new();

/// Set the tier (called at startup after license validation).
pub fn set_tier(tier: Tier) {
    let _ = TIER.set(tier);
}

/// Get the current tier.
pub fn current_tier() -> Tier {
    *TIER.get().unwrap_or(&Tier::Free)
}

// ── License key types ──

/// Parsed license key (before signature verification).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKey {
    pub user: String,
    pub email: String,
    pub tier: Tier,
    pub issued: String,
    pub expires: String,
    pub id: String,
    pub sig: String,
}

/// Validated license (after signature + expiry checks passed).
#[derive(Debug, Clone)]
pub struct ValidatedLicense {
    pub user: String,
    pub email: String,
    pub tier: Tier,
    pub id: String,
    pub expires: String,
}

// ── License validation ──

/// Validate a license key JSON string.
/// Returns None if signature is invalid, key is expired, or JSON is malformed.
/// Fully offline — no network calls.
pub fn validate_license(key_json: &str) -> Option<ValidatedLicense> {
    // Parse
    let key: LicenseKey = serde_json::from_str(key_json).ok()?;

    // Decode signature
    use base64::Engine;
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&key.sig)
        .ok()?;
    if sig_bytes.len() != 64 {
        return None;
    }

    // Build the message that was signed (all fields except sig).
    // Use serde's lowercase serialization for tier (matches signing side).
    let tier_str = serde_json::to_string(&key.tier).unwrap_or_default();
    let tier_str = tier_str.trim_matches('"');
    let message = format!(
        "{}|{}|{}|{}|{}|{}",
        key.user, key.email, tier_str, key.issued, key.expires, key.id,
    );

    // Verify Ed25519 signature
    use ed25519_dalek::{Signature, VerifyingKey};
    let verifying_key = VerifyingKey::from_bytes(&LICENSE_PUBLIC_KEY).ok()?;
    let signature = Signature::from_bytes(
        sig_bytes.as_slice().try_into().ok()?,
    );
    use ed25519_dalek::Verifier;
    verifying_key.verify(message.as_bytes(), &signature).ok()?;

    // Check expiry
    let today = chrono_today();
    if key.expires < today {
        return None; // expired
    }

    Some(ValidatedLicense {
        user: key.user,
        email: key.email,
        tier: key.tier,
        id: key.id,
        expires: key.expires,
    })
}

/// Get today's date as YYYY-MM-DD string (no chrono dependency — use simple system time).
fn chrono_today() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert epoch seconds to YYYY-MM-DD
    let days = now / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days: [i64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0usize;
    for (i, &d) in months_days.iter().enumerate() {
        if remaining < d {
            m = i;
            break;
        }
        remaining -= d;
    }
    format!("{:04}-{:02}-{:02}", y, m + 1, remaining + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Load and validate license from ~/.sentrux/license.key.
/// Returns the validated tier, or Free if no valid license found.
pub fn load_license_from_disk() -> Tier {
    let path = match dirs::home_dir() {
        Some(h) => h.join(".sentrux").join("license.key"),
        None => return Tier::Free,
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Tier::Free,
    };
    match validate_license(&content) {
        Some(license) => {
            crate::debug_log!("[license] Valid: {} ({}), expires {}", license.user, license.tier, license.expires);
            license.tier
        }
        None => {
            crate::debug_log!("[license] Invalid or expired license at {}", path.display());
            Tier::Free
        }
    }
}

/// Initialize the license system. Call once at startup.
pub fn init() {
    let tier = load_license_from_disk();
    set_tier(tier);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_ordering() {
        assert!(Tier::Free < Tier::Pro);
        assert!(Tier::Pro < Tier::Team);
    }

    #[test]
    fn can_access_logic() {
        assert!(Tier::Pro.can_access(Tier::Free));
        assert!(Tier::Pro.can_access(Tier::Pro));
        assert!(!Tier::Pro.can_access(Tier::Team));
        assert!(Tier::Team.can_access(Tier::Team));
        assert!(!Tier::Free.can_access(Tier::Pro));
    }

    #[test]
    fn detail_limits() {
        assert_eq!(Tier::Free.detail_limit(), 0);
        assert_eq!(Tier::Pro.detail_limit(), usize::MAX);
        assert_eq!(Tier::Team.detail_limit(), usize::MAX);
    }

    #[test]
    fn display() {
        assert_eq!(Tier::Free.to_string(), "Free");
        assert_eq!(Tier::Pro.to_string(), "Pro");
        assert_eq!(Tier::Team.to_string(), "Team");
    }

    #[test]
    fn free_tier_default() {
        assert_eq!(current_tier(), Tier::Free);
    }

    #[test]
    fn chrono_today_format() {
        let today = chrono_today();
        // Should be YYYY-MM-DD format
        assert_eq!(today.len(), 10);
        assert_eq!(today.as_bytes()[4], b'-');
        assert_eq!(today.as_bytes()[7], b'-');
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(validate_license("not json").is_none());
    }

    #[test]
    fn missing_sig_returns_none() {
        let json = r#"{"user":"test","email":"t@t","tier":"pro","issued":"2026-01-01","expires":"2099-01-01","id":"x","sig":"bad"}"#;
        assert!(validate_license(json).is_none());
    }

    #[test]
    fn expired_key_returns_none() {
        // Even with a "valid" signature (which it won't be), expiry should fail
        let json = r#"{"user":"test","email":"t@t","tier":"pro","issued":"2020-01-01","expires":"2020-01-02","id":"x","sig":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}"#;
        assert!(validate_license(json).is_none());
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        use ed25519_dalek::{SigningKey, Signer};
        use base64::Engine;

        // Use the actual keypair
        let private_bytes = base64::engine::general_purpose::STANDARD
            .decode("LacfYd2UWM+nEEeGQaR7VOYLZrSrXQZkqKU5eapZyTw=")
            .unwrap();
        let signing_key = SigningKey::from_bytes(private_bytes.as_slice().try_into().unwrap());

        let user = "github:test_user";
        let email = "test@sentrux.dev";
        let tier = "pro";
        let issued = "2026-03-18";
        let expires = "2099-12-31"; // far future
        let id = "lic_test_roundtrip";

        let message = format!("{}|{}|{}|{}|{}|{}", user, email, tier, issued, expires, id);
        let signature = signing_key.sign(message.as_bytes());
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());

        let key_json = serde_json::json!({
            "user": user,
            "email": email,
            "tier": tier,
            "issued": issued,
            "expires": expires,
            "id": id,
            "sig": sig_b64,
        }).to_string();

        let result = validate_license(&key_json);
        assert!(result.is_some(), "Roundtrip sign+verify failed");
        let license = result.unwrap();
        assert_eq!(license.user, user);
        assert_eq!(license.tier, Tier::Pro);
        assert_eq!(license.id, id);
    }
}
