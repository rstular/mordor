use std::collections::HashMap;

use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use serde::{Deserialize, Serialize};

pub(crate) type SessionState = HashMap<String, String>;

#[derive(Serialize, Deserialize)]
struct SessionStateWrapper {
    expiration: time::OffsetDateTime,
    state: SessionState,
}

#[non_exhaustive]
#[derive(Default)]
pub struct CookieTTLSessionStore;

impl CookieTTLSessionStore {
    async fn update_simple(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        self.save(session_state, ttl)
            .await
            .map_err(|err| match err {
                SaveError::Serialization(err) => UpdateError::Serialization(err),
                SaveError::Other(err) => UpdateError::Other(err),
            })
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for CookieTTLSessionStore {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        let state: SessionStateWrapper = serde_json::from_str(session_key.as_ref())
            .map_err(anyhow::Error::new)
            .map_err(LoadError::Deserialization)?;

        if state.expiration < time::OffsetDateTime::now_utc() {
            return Ok(None);
        }

        Ok(Some(state.state))
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let wrapper = SessionStateWrapper {
            expiration: time::OffsetDateTime::now_utc() + *ttl,
            state: session_state,
        };

        let session_key = serde_json::to_string(&wrapper)
            .map_err(anyhow::Error::new)
            .map_err(SaveError::Serialization)?;

        Ok(session_key
            .try_into()
            .map_err(Into::into)
            .map_err(SaveError::Other)?)
    }

    async fn update(
        &self,
        _session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        self.update_simple(session_state, ttl).await
    }

    async fn update_ttl(
        &self,
        session_key: &SessionKey,
        ttl: &Duration,
    ) -> Result<(), anyhow::Error> {
        let state = if let Some(state) = self.load(session_key).await? {
            state
        } else {
            HashMap::new()
        };

        self.update_simple(state, ttl).await?;
        Ok(())
    }

    async fn delete(&self, _session_key: &SessionKey) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
