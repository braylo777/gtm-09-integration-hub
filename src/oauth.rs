//! OAuth 2.0 authorization code + PKCE (RFC 6749, RFC 7636) validation. Every check is listed
//! with pass / fail and a reason, so an integration partner can see exactly what is wrong.
use crate::util::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub fn b64url(b: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut o = String::new();
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for k in 0..=c.len() {
            o.push(A[(n >> (18 - 6 * k) & 63) as usize] as char);
        }
    }
    o
}

pub fn s256(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}

fn verifier_ok(v: &str) -> bool {
    (43..=128).contains(&v.len()) && v.bytes().all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
}

pub fn check_op(i: &Value) -> R<Value> {
    let client = i.get("client").ok_or("`client` is required")?;
    let auth = i.get("authorize").ok_or("`authorize` is required")?;
    let cb = i.get("callback").ok_or("`callback` is required")?;
    let tok = i.get("token_request").ok_or("`token_request` is required")?;
    let mut checks = vec![];
    let mut add = |name: &str, ok: bool, why: String| checks.push(json!({"check": name, "pass": ok, "detail": why}));
    let cid = s(client, "client_id")?;
    add("client_id matches", s_or(auth, "client_id", "") == cid, format!("authorize.client_id = {}", s_or(auth, "client_id", "")));
    let uris: Vec<&str> = arr(client, "redirect_uris")?.iter().filter_map(Value::as_str).collect();
    let ru = s_or(auth, "redirect_uri", "");
    add("redirect_uri registered (exact match)", uris.contains(&ru), format!("{ru}"));
    add("redirect_uri uses https (or loopback)", ru.starts_with("https://") || ru.starts_with("http://127.0.0.1") || ru.starts_with("http://localhost"), ru.to_string());
    let allowed: Vec<&str> = arr(client, "allowed_scopes")?.iter().filter_map(Value::as_str).collect();
    let asked: Vec<&str> = s_or(auth, "scope", "").split_whitespace().collect();
    let extra: Vec<&str> = asked.iter().copied().filter(|x| !allowed.contains(x)).collect();
    add("scopes within allowed set", extra.is_empty() && !asked.is_empty(), if extra.is_empty() { format!("{}", asked.join(" ")) } else { format!("not allowed: {}", extra.join(" ")) });
    let st = s_or(auth, "state", "");
    add("state present (≥ 16 chars)", st.len() >= 16, format!("{} chars", st.len()));
    add("state matches on callback", !st.is_empty() && s_or(cb, "state", "") == st, "CSRF protection".into());
    add("code_challenge_method is S256", s_or(auth, "code_challenge_method", "") == "S256", s_or(auth, "code_challenge_method", "(missing)").to_string());
    let v = s_or(tok, "code_verifier", "");
    add("code_verifier format (43–128 unreserved chars)", verifier_ok(v), format!("{} chars", v.len()));
    let calc = s256(v);
    add("S256(code_verifier) == code_challenge", calc == s_or(auth, "code_challenge", ""), format!("computed {calc}"));
    add("token redirect_uri equals authorize redirect_uri", s_or(tok, "redirect_uri", "") == ru, s_or(tok, "redirect_uri", "").to_string());
    let issued = f_or(cb, "code_issued_at", 0.0);
    let used = f_or(tok, "sent_at", 0.0);
    add("code used within 60 s and once", used >= issued && used - issued <= 60.0 && !tok.get("code_already_used").and_then(Value::as_bool).unwrap_or(false), format!("{} s after issue", used - issued));
    let pass = checks.iter().all(|c| c["pass"] == true);
    let exp = f_or(i, "expires_in", 3600.0);
    let skew = f_or(i, "refresh_skew_sec", 300.0).min(exp / 2.0);
    Ok(json!({"flow_valid": pass, "failed": checks.iter().filter(|c| c["pass"] == false).count(), "checks": checks,
        "token_plan": {"expires_in_sec": exp, "refresh_at_sec": exp - skew, "note": "refresh before expiry; store refresh tokens encrypted; rotate on each use"}}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn rfc7636_vector() {
        // RFC 7636 appendix B
        assert_eq!(s256("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        assert_eq!(b64url(b"f"), "Zg");
        assert_eq!(b64url(b"foo"), "Zm9v");
    }
    #[test]
    fn flags_scope_and_pkce() {
        let v = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let i = json!({"client":{"client_id":"c","redirect_uris":["https://a/cb"],"allowed_scopes":["read"]},
            "authorize":{"client_id":"c","redirect_uri":"https://a/cb","scope":"read write","state":"0123456789abcdef","code_challenge":"wrong","code_challenge_method":"S256"},
            "callback":{"state":"0123456789abcdef","code_issued_at":0},"token_request":{"code_verifier":v,"redirect_uri":"https://a/cb","sent_at":5}});
        let r = check_op(&i).unwrap();
        assert_eq!(r["flow_valid"], false);
        assert_eq!(r["failed"], 2);
    }
}
