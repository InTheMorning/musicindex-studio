//! The export of an account (ADR 0107 items 3 to 8) and the recovery key
//! that it can need (ADR 0107 item 4).

use std::time::Instant;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use studio_core::account_archive::archive_sizes;
use studio_core::vault::VaultMethod;

use crate::error::ApiError;
use crate::routes::{ApiResult, Shared, blocking};
use crate::workspace::Workspace;

/// `POST /vault/recovery-key`: make a new recovery key. It replaces an
/// older key. The response holds the key one time, and nothing else
/// returns it. The route needs a recent unlock (ADR 0106 item 2).
pub(crate) async fn recovery_key(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let key = w.new_recovery_key()?;
        Ok(Json(json!({ "recovery_key": key.as_str() })))
    })
    .await
}

/// True when a method of the vault opens it on a different server: a
/// passphrase, a recovery key or a Nostr signer (ADR 0107 item 4).
fn movable(methods: &[VaultMethod]) -> bool {
    methods.iter().any(|m| {
        matches!(
            m,
            VaultMethod::Passphrase | VaultMethod::Recovery | VaultMethod::Nostr { .. }
        )
    })
}

/// The kinds of the methods, once each, in a fixed sequence.
fn method_names(methods: &[VaultMethod]) -> Vec<&'static str> {
    let has = |test: fn(&VaultMethod) -> bool| methods.iter().any(test);
    [
        ("passphrase", has(|m| matches!(m, VaultMethod::Passphrase))),
        ("recovery", has(|m| matches!(m, VaultMethod::Recovery))),
        ("nostr", has(|m| matches!(m, VaultMethod::Nostr { .. }))),
        ("passkey", has(|m| matches!(m, VaultMethod::Passkey { .. }))),
    ]
    .into_iter()
    .filter_map(|(name, found)| found.then_some(name))
    .collect()
}

/// The library folder of the workspace, for the archive. A library
/// outside the data folder goes into `library/` of the archive.
fn library_of(w: &Workspace) -> Result<Option<std::path::PathBuf>, ApiError> {
    Ok(w.collection.library_root()?)
}

/// `GET /account/export-facts`: what the export sheet shows (ADR 0107
/// items 4 and 5). It needs no unlock.
pub(crate) async fn export_facts(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        let methods = w.vault_methods()?;
        let library = library_of(w)?;
        let sizes = archive_sizes(&w.data_dir, library.as_deref())
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let counts = w.collection.account_counts()?;
        // A server that holds no assets keeps only the files that wait for
        // a write. They go in the archive, and the user has no choice.
        let holds = w.holds_assets();
        let default_bytes = sizes.with_assets;
        Ok(Json(json!({
            "feeds": counts.feeds,
            "drafts": counts.drafts,
            "versions": counts.versions,
            "people": counts.people,
            "movable": movable(&methods),
            "methods": method_names(&methods),
            "passkeys": methods.iter().filter(|m| matches!(m, VaultMethod::Passkey { .. })).count(),
            "holds_assets": holds,
            "asset_bytes": sizes.with_assets.saturating_sub(sizes.without_assets),
            "archive_bytes": default_bytes,
            "archive_bytes_without_assets": sizes.without_assets,
        })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct ExportRequest {
    /// With `false`, the archive holds the feed XML files and no other
    /// library file. A server with `--hold-assets off` ignores the field.
    #[serde(default = "yes")]
    include_assets: bool,
}

fn yes() -> bool {
    true
}

/// `POST /account/export`: make a media ticket for the archive (ADR 0107
/// items 3, 4 and 7). The route needs a recent unlock. A vault that no
/// different server can open gives 409 `vault_not_movable`. The ticket
/// works one time, on `GET /media/download`.
pub(crate) async fn export(
    State(shared): State<Shared>,
    Json(body): Json<ExportRequest>,
) -> ApiResult {
    blocking(shared, move |w| {
        w.require_recent_unlock()?;
        if !movable(&w.vault_methods()?) {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "vault_not_movable",
                "ADR 0107 item 4: only passkeys open this vault, and a passkey does not work on a different server. Make a recovery key first",
            ));
        }
        let include_assets = if w.holds_assets() {
            body.include_assets
        } else {
            true
        };
        let sizes = archive_sizes(&w.data_dir, library_of(w)?.as_deref())
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let ticket = w.tickets.issue_archive(include_assets, Instant::now())?;
        Ok(Json(json!({
            "ticket": ticket,
            "file_name": archive_name(crate::announcer::now()),
            "include_assets": include_assets,
            "archive_bytes": if include_assets { sizes.with_assets } else { sizes.without_assets },
        })))
    })
    .await
}

/// The file name of an archive: `studio-` and the date of the export.
pub(crate) fn archive_name(unix_seconds: u64) -> String {
    // The civil date of a day number (Howard Hinnant, `civil_from_days`).
    let days = i64::try_from(unix_seconds / 86_400).unwrap_or(0) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("studio-{year:04}-{month:02}-{day:02}.tar")
}

#[cfg(test)]
mod tests {
    use super::archive_name;

    #[test]
    fn the_archive_name_has_the_date_of_the_export() {
        // 2026-11-01 00:00:00 UTC, and the last second of 2024-02-29.
        assert_eq!(archive_name(1_793_491_200), "studio-2026-11-01.tar");
        assert_eq!(archive_name(1_709_251_199), "studio-2024-02-29.tar");
        assert_eq!(archive_name(0), "studio-1970-01-01.tar");
    }
}
