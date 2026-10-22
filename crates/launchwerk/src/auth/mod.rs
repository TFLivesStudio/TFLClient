// Copyright (C) 2026 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
// src/auth/mod.rs
#![cfg(feature = "auth")]
pub mod microsoft;
pub mod oauth;
pub mod storage;
#[cfg(test)]
mod tests;
pub mod yggdrasil;

use serde::{Deserialize, Serialize};
use storage::SecureStorage;

#[derive(Clone, Serialize, Deserialize)]
pub struct MinecraftUser {
    pub username: String,
    pub uuid: String,
    #[serde(skip)]
    pub access_token: String,
    #[serde(skip)]
    pub refresh_token: Option<String>,
    #[serde(skip)]
    pub access_token_expires_at: Option<u64>,
    pub user_type: AccountType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub microsoft_client_id: Option<String>,
    #[serde(default)]
    pub session_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yggdrasil_server_url: Option<String>,
    #[serde(skip)]
    pub client_token: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Cracked,
    Microsoft,
    Yggdrasil,
}

impl MinecraftUser {
    /// Create a new cracked (offline) user
    pub fn cracked(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            uuid: uuid::Uuid::new_v4().to_string(),
            access_token: "0".to_string(),
            refresh_token: None,
            access_token_expires_at: None,
            microsoft_client_id: None,
            session_only: false,
            user_type: AccountType::Cracked,
            yggdrasil_server_url: None,
            client_token: None,
        }
    }

    /// Create a new premium user (from Microsoft auth)
    pub fn premium(
        username: String,
        uuid: String,
        access_token: String,
        refresh_token: Option<String>,
    ) -> Self {
        Self {
            username,
            uuid,
            access_token,
            refresh_token,
            access_token_expires_at: None,
            microsoft_client_id: None,
            session_only: false,
            user_type: AccountType::Microsoft,
            yggdrasil_server_url: None,
            client_token: None,
        }
    }

    /// Create a new Yggdrasil user
    pub fn yggdrasil(
        username: String,
        uuid: String,
        access_token: String,
        client_token: String,
        server_url: String,
    ) -> Self {
        Self {
            username,
            uuid,
            access_token,
            refresh_token: None,
            access_token_expires_at: None,
            microsoft_client_id: None,
            session_only: false,
            user_type: AccountType::Yggdrasil,
            yggdrasil_server_url: Some(server_url),
            client_token: Some(client_token),
        }
    }

    pub fn requires_reauthorization(&self, client_id: &str) -> bool {
        self.user_type == AccountType::Microsoft
            && self.microsoft_client_id.as_deref() != Some(client_id)
    }
    pub fn cached_session_valid_at(&self, now: u64) -> bool {
        self.user_type == AccountType::Microsoft
            && !self.access_token.is_empty()
            && self
                .access_token_expires_at
                .is_some_and(|expiry| expiry > now.saturating_add(60))
    }
    fn provider(&self) -> String {
        match self.user_type {
            AccountType::Microsoft => "microsoft".into(),
            AccountType::Yggdrasil => format!(
                "yggdrasil:{}",
                self.yggdrasil_server_url.as_deref().unwrap_or("")
            ),
            AccountType::Cracked => "offline".into(),
        }
    }

    /// No plaintext tokens are serialized; the OS vault protects a random envelope key.
    pub fn save_tokens(&self) -> crate::Result<()> {
        if self.user_type == AccountType::Cracked || self.session_only {
            return Ok(());
        }
        let record = storage::SessionSecrets {
            version: 2,
            expires_at: self.access_token_expires_at,
            access: self.access_token.clone(),
            refresh: self.refresh_token.clone(),
            client_token: self.client_token.clone(),
            issuer: if self.user_type == AccountType::Microsoft {
                Some(
                    self.microsoft_client_id
                        .clone()
                        .unwrap_or_else(|| microsoft::DEFAULT_CLIENT_ID.into()),
                )
            } else {
                None
            },
        };
        SecureStorage::save(&self.provider(), &self.uuid, &record)?;
        // Read-back has succeeded before any old credentials are removed.
        SecureStorage::delete_legacy(&self.uuid)
    }

    pub fn load_tokens(&mut self) -> crate::Result<()> {
        if self.user_type == AccountType::Cracked || !self.access_token.is_empty() {
            return Ok(());
        }
        if self.session_only {
            return Err(crate::Error::AuthError("REAUTHENTICATION_REQUIRED".into()));
        }
        let record = match SecureStorage::load(&self.provider(), &self.uuid)? {
            Some(record) => record,
            None => {
                let access = SecureStorage::legacy(&self.uuid, "access")?
                    .ok_or_else(|| crate::Error::AuthError("REAUTHENTICATION_REQUIRED".into()))?;
                let record = storage::SessionSecrets {
                    version: 2,
                    expires_at: None,
                    access,
                    refresh: SecureStorage::legacy(&self.uuid, "refresh")?,
                    client_token: SecureStorage::legacy(&self.uuid, "client")?,
                    issuer: if self.user_type == AccountType::Microsoft {
                        Some(microsoft::DEFAULT_CLIENT_ID.into())
                    } else {
                        None
                    },
                };
                let record = SecureStorage::migrate(&self.provider(), &self.uuid, record)?;
                // Failure preserves the account and old files; never silently downgrades.
                SecureStorage::delete_legacy(&self.uuid)?;
                record
            }
        };
        self.access_token_expires_at = record.expires_at;
        self.access_token = record.access.clone();
        self.refresh_token = record.refresh.clone();
        self.client_token = record.client_token.clone();
        self.microsoft_client_id = record.issuer.clone();
        Ok(())
    }

    pub fn delete_tokens(&self) -> crate::Result<()> {
        if self.user_type == AccountType::Cracked {
            return Ok(());
        }
        SecureStorage::delete(&self.provider(), &self.uuid)
    }
}

impl std::fmt::Debug for MinecraftUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinecraftUser")
            .field("uuid", &self.uuid)
            .field("user_type", &self.user_type)
            .field("credentials", &"[REDACTED]")
            .finish()
    }
}
