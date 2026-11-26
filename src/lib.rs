//! Stage 9 — Integration-Led Growth (Ecosystem GTM).
//!
//! * `oauth`     — check an OAuth 2.0 authorization-code flow with PKCE (S256) step by step:
//!   client, redirect URI, scopes, state, code challenge, token expiry and refresh timing.
//! * `templates` — recommend Zapier / Make-style templates from the apps a customer has
//!   connected: ready now vs one connection away.
//! * `ecosystem` — partner-sourced and partner-influenced revenue, co-sell table, and win
//!   rate with vs without an integration in use.

mod api;
pub mod ecosystem;
pub mod oauth;
pub mod templates;
pub mod util;

pub use api::dispatch;
use serde_json::Value;

pub const OPS: &[&str] = &["oauth_check", "recommend_templates", "ecosystem_revenue"];

pub fn handle(op: &str, i: &Value) -> Result<Value, String> {
    match op {
        "oauth_check" => oauth::check_op(i),
        "recommend_templates" => templates::recommend_op(i),
        "ecosystem_revenue" => ecosystem::revenue_op(i),
        _ => Err(format!("unknown op `{op}`; known: {}", OPS.join(", "))),
    }
}
