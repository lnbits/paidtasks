# PaidTasks — Agent Notes (Template)

This extension is a **template** for WASM-only community extensions.
No Python code from the extension is executed by LNbits.

## What This Template Is
- A starting point you copy and change.
- A reference for permissions, proxy usage, and the KV schema.

## What It Can Do (When Permissions Are Granted)
- Read/write the extension KV store (`ext.db.read_write`).
- Read/write secret KV via `db_secret_get/db_secret_set` (not exposed to public pages).
- Call internal LNbits endpoints via host HTTP (`api.METHOD:/path`).
- Publish websocket events via `ws_publish` (namespace must start with `ext_id:`).
- Register backend payment watchers (`ext.payments.watch`).

Any internal LNbits API endpoint can be called **only if** it is allowlisted in `config.json`:
- Add `api.METHOD:/path` to the `permissions` list.
- Then call it from WASM via `http_request` with the same method + path.

## Clone Checklist (Agent‑Friendly)
1. Keep `id` consistent:
- `manifest.json` `id`
- `config.json` `id`
- folder name (if you rename it)

2. Permissions:
- Add only the `api.METHOD:/path` you need.
- Add all public handlers to `public_wasm_functions`.
- Keep `public_kv_keys` minimal.

3. KV schema:
- Add all keys to `kv_schema`.
- Secrets live only in secret KV (`db_secret_*`).

4. WASM:
- Export functions with exact names used by frontend.
- Avoid unbounded loops or large allocations.

5. UI:
- Public pages should not rely on `window.g` or `LNbits` globals.
- Public actions must call allowlisted public handlers.

## What It Cannot Do
- Access the filesystem.
- Access environment variables or LNbits settings (outside of the API for fetching settings).
- Make outbound network calls.
- Execute Python or shell commands.

## Files You’ll Change
- `config.json`: permissions, public handlers, and KV allowlist.
- `wasm/`: your WASM module (WAT/Rust/AssemblyScript examples provided).
- `static/` + `templates/`: your UI.

## How to Vibe-Change This Template
1. Update permissions in `config.json` to match the APIs you need.
2. Define KV keys and defaults in `kv_schema`.
3. Set `public_wasm_functions` for any public handlers.
4. Replace the UI with your own pages.
5. Build your WASM module and replace `wasm/module.wasm`.

## Limits
- WASM execution is time-limited by `LNBITS_WASM_TIMEOUT_SECONDS`.
- Optional fuel limit `LNBITS_WASM_FUEL` (can be disabled by setting to 0).
- KV writes are validated against `kv_schema` when present.
- HTTP calls are permission-checked (`api.METHOD:/path`).
- Public handlers must be allowlisted in `public_wasm_functions`.
