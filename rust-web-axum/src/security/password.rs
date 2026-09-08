use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha512};
use thiserror::Error;

/// Minimum zxcvbn score required (0–4 scale).
/// 3 = "safely unguessable", 4 = "very unguessable".
const MIN_ZXCVBN_SCORE: u8 = 3;

#[derive(Debug, Error)]
#[error("password too weak: {reason}")]
pub struct PasswordStrengthError {
    pub reason: String,
}

/// Validate password strength using the zxcvbn algorithm (Dropbox's password
/// strength estimator). Rejects passwords scoring below [`MIN_ZXCVBN_SCORE`].
///
/// `user_inputs` should contain context strings (username, email, etc.) that
/// the estimator can penalise if they appear in the password.
pub fn validate_password_strength(
    password: &str,
    user_inputs: &[&str],
) -> Result<(), PasswordStrengthError> {
    let entropy = zxcvbn::zxcvbn(password, user_inputs);
    let score = entropy.score();

    if (score as u8) < MIN_ZXCVBN_SCORE {
        let feedback_msg = entropy
            .feedback()
            .as_ref()
            .and_then(|f| f.warning())
            .map(|w| format!("{}", w))
            .unwrap_or_default();

        let suggestion = entropy
            .feedback()
            .as_ref()
            .map(|f| {
                f.suggestions()
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();

        let mut reason = format!(
            "score {}/4, minimum required {}/4",
            score as u8, MIN_ZXCVBN_SCORE
        );
        if !feedback_msg.is_empty() {
            reason.push_str(&format!(". {}", feedback_msg));
        }
        if !suggestion.is_empty() {
            reason.push_str(&format!(". Suggestions: {}", suggestion));
        }

        return Err(PasswordStrengthError { reason });
    }

    Ok(())
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("operating-system randomness must be available for password hashing")
        .to_string()
}

/// Verify current Argon2id hashes and the previous `sha512$...` representation.
/// Plaintext storage is intentionally never accepted. Successful legacy logins
/// can be re-hashed by the caller during a controlled credential migration.
pub fn verify_password(plain_password: &str, stored_hash: &str) -> bool {
    if stored_hash.starts_with("$argon2") {
        return PasswordHash::new(stored_hash).ok().is_some_and(|parsed| {
            Argon2::default()
                .verify_password(plain_password.as_bytes(), &parsed)
                .is_ok()
        });
    }

    verify_legacy_sha512(plain_password, stored_hash)
}

fn verify_legacy_sha512(plain_password: &str, stored_hash: &str) -> bool {
    let Some(stripped) = stored_hash.strip_prefix("sha512$") else {
        return false;
    };
    let mut hasher = Sha512::new();
    hasher.update(plain_password.as_bytes());
    let hex_hash = hex::encode(hasher.finalize());
    hex_hash == stripped
}
