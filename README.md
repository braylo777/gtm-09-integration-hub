# gtm-09-integration-hub

**GTM Engineering — Stage 9 of 12: Integration-Led Growth (Ecosystem GTM)**

Grow through the tools customers already use: a step-by-step OAuth 2.0 + PKCE flow checker, Zapier / Make-style template recommendations from connected apps, and partner-sourced vs influenced revenue with integration win-rate lift.

**Live demo:** https://gtm-09-integration-hub.braylo777.workers.dev — the Rust core runs in your browser as WebAssembly. No server, no tracking.

## Why this matters

Customers who connect three integrations churn less and buy more, and partners bring deals a sales team cannot reach alone. But marketplace listings stall on OAuth bugs, customers never find the template that fits their stack, and partner revenue is argued about in every QBR. This repo makes the flow checkable, the next template obvious, and partner credit countable.

## What it does

- OAuth 2.0 authorization code + PKCE (RFC 7636 S256), checked step by step: client id, exact redirect URI, https, scopes, state length and match, verifier format, challenge, code reuse and age
- SHA-256 + base64url verified against the RFC 7636 appendix B test vector
- Token refresh plan: refresh before expiry with a skew
- Template recommendations: ready now vs one connection away, goal-tag match, installs; the one app that unlocks the most templates
- Ecosystem revenue: partner-sourced vs influenced-only, per-partner co-sell table, win rate with vs without an integration in use

## What it does not do (yet)

- Checks a recorded flow; it is not an authorization server and issues no tokens
- No Zapier / Make API: the catalog is synthetic
- Integration lift is a correlation on closed deals, not a causal estimate

## Operations

Every operation takes JSON in and gives JSON out. The CLI, the tests and the demo page all use the same entry point.

| Operation | What it does |
|---|---|
| `oauth_check` | A correct authorization-code + PKCE flow. The verifier and challenge are the RFC 7636 test vector. |
| `recommend_templates` | Recommend automation templates for a customer who connected HubSpot and Slack and cares about routing and churn. |
| `ecosystem_revenue` | Split won revenue into partner-sourced and influenced, and compare win rates with and without an integration. |

## Run it

You need Rust (stable). No Node, no npm.

```bash
git clone https://github.com/braylo777/gtm-09-integration-hub
cd gtm-09-integration-hub
cargo test
cargo run -- ops
```

Try the first demo example from the command line:

```bash
python3 -c 'import json;print(json.dumps(json.load(open("web/examples.json"))["ops"][0]["input"]))' | cargo run -q -- oauth_check
```

## Build and deploy the demo

```bash
./build.sh    # tests, then compiles the core to web/app.wasm
./deploy.sh   # serves web/ as a Cloudflare Workers static-assets site (needs a Cloudflare login)
```

`web/` is a plain static site (`index.html`, `app.js`, `app.wasm`). It has no build step, so any static host works.

## Layout

```
src/lib.rs        operation list and router
src/api.rs        JSON entry point + the 3 WASM exports (alloc, dealloc, run)
src/main.rs       CLI
src/*.rs          one module per capability, each with unit tests
tests/            integration tests (every demo example must pass)
web/              demo page; examples.json drives both the page and this README
```

## Design choices

- **Rust, few dependencies.** Only `serde_json` and `sha2`, pinned to exact versions with a committed `Cargo.lock`.
- **Deterministic.** Time, randomness and keys are inputs, so every result can be replayed in a test or an audit.
- **Fail closed.** Unknown input or missing fields give an error, never a silent default.

## Part of a series

This is one of 12 repos that work through a public Go-To-Market (GTM) Engineering roadmap: [12 stages by Suraj Sharma (@suraj_sharma14)](https://x.com/suraj_sharma14/status/2125552583452270692). The roadmap is the inspiration; the code here is original. All 12: [github.com/braylo777?q=gtm-](https://github.com/braylo777?tab=repositories&q=gtm-).

## Data and privacy

Every example uses synthetic data. Nothing calls a paid API, scrapes a site, identifies a real visitor or sends an email.

## License

MIT — see [LICENSE](LICENSE). Built by [Braylo SI](https://braylo.si).
