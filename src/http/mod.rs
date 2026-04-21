pub mod account;
pub mod perps;
pub mod portfolio;
pub mod swap;
pub mod tasks;
pub mod tokens;
pub mod trigger;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::error::ShurikenError;

const DEFAULT_BASE_URL: &str = "https://api.shuriken.trade";

#[derive(Clone)]
pub struct ShurikenHttpClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
}

impl ShurikenHttpClient {
    pub fn new(api_key: &str) -> Result<Self, ShurikenError> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self, ShurikenError> {
        let mut headers = HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|e| ShurikenError::Auth(e.to_string()))?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);

        let http = Client::builder().default_headers(headers).build()?;
        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self { http, base_url })
    }

    // ── Namespace accessors ────────────────────────────────────────────────

    pub fn account(&self) -> account::AccountApi<'_> {
        account::AccountApi(self)
    }

    pub fn tokens(&self) -> tokens::TokensApi<'_> {
        tokens::TokensApi(self)
    }

    pub fn swap(&self) -> swap::SwapApi<'_> {
        swap::SwapApi(self)
    }

    pub fn portfolio(&self) -> portfolio::PortfolioApi<'_> {
        portfolio::PortfolioApi(self)
    }

    pub fn trigger(&self) -> trigger::TriggerApi<'_> {
        trigger::TriggerApi(self)
    }

    pub fn perps(&self) -> perps::PerpsApi<'_> {
        perps::PerpsApi(self)
    }

    pub fn tasks(&self) -> tasks::TasksApi<'_> {
        tasks::TasksApi(self)
    }

    // ── HTTP helpers ───────────────────────────────────────────────────────

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ShurikenError> {
        let resp = self.http.get(self.url(path)).send().await?;
        self.handle_response(resp).await
    }

    pub(crate) async fn get_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, ShurikenError> {
        let resp = self.http.get(self.url(path)).query(query).send().await?;
        self.handle_response(resp).await
    }

    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, ShurikenError> {
        let resp = self.http.post(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    pub(crate) async fn put<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, ShurikenError> {
        let resp = self.http.put(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    pub(crate) async fn patch<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, ShurikenError> {
        let resp = self.http.patch(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    pub(crate) async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ShurikenError> {
        let resp = self.http.delete(self.url(path)).send().await?;
        self.handle_response(resp).await
    }

    pub(crate) async fn delete_with_body<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, ShurikenError> {
        let resp = self.http.delete(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, ShurikenError> {
        let status = resp.status();
        let request_id = resp
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if status == reqwest::StatusCode::UNAUTHORIZED {
            let text = resp.text().await.unwrap_or_default();
            return Err(ShurikenError::Auth(text));
        }

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ShurikenError::Api {
                status: status.as_u16(),
                message: text,
                request_id,
            });
        }

        let body: serde_json::Value = resp.json().await?;
        let payload = match body.get("data") {
            Some(data) => data.clone(),
            None => body,
        };
        Ok(serde_json::from_value(payload)?)
    }
}
