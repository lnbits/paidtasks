# PaidTasks (WASM-only)

Create paid task lists with public share links.

## Features
- Create lists and tasks with a cost (sats).
- Share a public page per list where anyone can pay for a task.
- Task payments update the backend via watcher keys.

## Safety Model
- No filesystem / env access
- No arbitrary outbound network
- No arbitrary DB access (only its own KV / secret KV)
- No Python execution
- Time‑boxed execution (`LNBITS_WASM_TIMEOUT_SECONDS`)
- Optional fuel limit (`LNBITS_WASM_FUEL`)

## Data Model (KV)
- `lists`: JSON array of lists (private)
- `tasks`: JSON array of tasks (private)
- `public_lists`: JSON array (public)
- `public_tasks`: JSON array (public)
- `task_paid:<task_id>`: payment JSON (private)
- `task_cost:<task_id>`: task cost (private)
- `task_list:<task_id>`: list id (private)

## Data Model (Secret KV)
- `list_wallet_inkey:<list_id>`: wallet invoice key (secret)

## Permissions
- `ext.db.read_write` for KV.
- `api.POST:/api/v1/payments` to create invoices.
- `ext.payments.watch` to mark tasks as paid.

## Using LNbits APIs (Allowlist Required)
WASM can call any **internal LNbits API endpoint**, but only if it is allowlisted in `config.json`.

How to enable an endpoint:
1. Add the permission string to `config.json` under `permissions`.
   - Format: `api.METHOD:/path`
   - Example: `api.POST:/api/v1/payments`
2. Call it from WASM using `http_request` with the same method + path.

If it’s not allowlisted, the call will be rejected.

## Example Prompt
Use this when asking an agent to adapt PaidTasks into a new extension:

```
You are modifying the LNbits WASM extension `paidtasks` as a base.

Rules:
- Only edit files inside `lnbits/extensions/paidtasks/`.
- Do not modify LNbits core code.
- Update `config.json` permissions and `public_wasm_functions` as needed.
- Use secret KV for any keys or tokens.
- Keep public pages read-only and use public WASM handlers for public actions.
- If you add public functionality, add a handler to `public_wasm_functions`.
- Update `wasm/module.wat` (or build WASM from Rust/AssemblyScript) to implement backend logic.
- Update `templates/` and `static/` for UI changes.
- Keep public pages read-only; all public actions must go through allowlisted public WASM handlers.
- Ensure WASM exports match the handler names used in the UI.
- Use `db_secret_get/db_secret_set` for secrets and never return them to the frontend.
- Use `http_request` only for allowlisted LNbits API endpoints in `config.json`.
- Remember: any internal endpoint is allowed **only if** you add `api.METHOD:/path` to `config.json`.
- Avoid long loops or unbounded work to prevent timeouts (see `LNBITS_WASM_TIMEOUT_SECONDS` and `LNBITS_WASM_FUEL`).

Goal:
Describe the new use case and implement it end-to-end (KV schema, WASM handlers, UI, public page).
```

## Clone Checklist (Fool‑Proof)
Follow this list exactly when cloning PaidTasks into a new extension:

1. Rename the extension:
- Update `manifest.json` `id`, `name`, and `short_description`.
- Update `config.json` `id` and `name`.
- Update `description.md` and `README.md` title.

2. Update the permission model:
- In `config.json`, add only the exact `api.METHOD:/path` routes you need.
- If you expose public actions, list every handler in `public_wasm_functions`.
- Keep `public_kv_keys` minimal (public‑safe data only).

3. Define your data schema:
- Add all KV keys to `kv_schema` in `config.json`.
- Prefix keys consistently (e.g., `item_*`, `public_*`).
- Store secrets only in secret KV (`db_secret_*`), never in public KV.

4. Wire the WASM backend:
- Ensure every public handler exists as an exported WASM function.
- Match handler names exactly between UI and WASM.
- Use `http_request` for internal API calls only when allowlisted.
- Avoid unbounded loops and large allocations.

5. Update the UI:
- `templates/` for HTML pages.
- `static/js/` for frontend logic.
- Do not assume LNbits globals exist on public pages; use only what you import or attach explicitly.

6. Public page safety:
- Never expose admin or invoice keys.
- Public pages must call only allowlisted public WASM handlers.
- Any state updates should be sent via `ws_publish` if needed by the UI.

7. Replace example content:
- Update labels, table columns, and dialogs.
- Remove PaidTasks‑specific text and KV keys.

## Quick Start Rename (Exact Files)
Keep the extension `id` **consistent everywhere**:
- `lnbits/extensions/paidtasks/manifest.json`: `id`, `name`, `short_description`
- `lnbits/extensions/paidtasks/config.json`: `id`, `name`
- `lnbits/extensions/paidtasks/description.md`: title and summary
- `lnbits/extensions/paidtasks/README.md`: title and references

After renaming, restart LNbits so it picks up the new extension.

## Handler Wiring Example
Goal: public button triggers backend logic.

1. Add handler to `config.json`:
- `public_wasm_functions`: `["public_do_thing"]`

2. Export from WASM:
- WASM must export a function named `public_do_thing`.

3. Call from public page:
- JS calls `/paidtasks/api/v1/public/call/public_do_thing` with JSON.

If any name mismatches, you’ll get a 404 or handler error.

## KV Naming Rules
- `public_*` keys: safe to expose on public pages.
- `secret_*` keys: store only via secret KV (`db_secret_*`).
- `item_*` keys: internal/private by default.
- Always add keys to `kv_schema` in `config.json`.

## Common Errors (And Fixes)
- 404 on public handler: add it to `public_wasm_functions` and recompile WASM.
- 401 on internal API call: add `api.METHOD:/path` to `config.json` permissions.
- Public UI crash: don’t rely on `window.g` or `LNbits` unless you define them.
- WASM “fuel consumed” or timeout: avoid long loops; keep work bounded.
- “KV key not allowed”: add it to `kv_schema` (or remove schema if you don’t need it).

## Minimal Test Checklist
1. Create a record in the backend UI.
2. Verify it appears in the public page (only public KV data).
3. Trigger a public action (invoice creation / public handler).
4. Verify backend state updates (paid flag or record update).
5. Confirm websockets update UI without polling.

## Copy‑Paste Templates
### `config.json` snippets
Add a new public handler and API permission:
```json
{
  "public_wasm_functions": ["public_do_thing"],
  "permissions": [
    "ext.db.read_write",
    "api.POST:/api/v1/payments"
  ],
  "kv_schema": [
    {"key": "items", "type": "array", "public": false},
    {"key": "public_items", "type": "array", "public": true}
  ],
  "public_kv_keys": ["public_items"]
}
```

### Public page JS call
```js
const res = await fetch(`/paidtasks/api/v1/public/call/public_do_thing`, {
  method: "POST",
  headers: {"Content-Type": "application/json"},
  body: JSON.stringify({ id: "example" })
});
const data = await res.json();
```

### WASM WAT handler skeleton
```wat
(func (export "public_do_thing")
  ;; 1) read JSON request from key "public_request"
  ;; 2) do work and write JSON response to key "public_response"
)
```

## Minimal Debug Rules (Avoid Common Failures)
- If a handler is 404: add it to `public_wasm_functions` and recompile WASM.
- If a handler errors in WASM: check exported function name and argument types.
- If HTTP calls fail with 401: ensure `api.METHOD:/path` is allowlisted and the backend fetches secrets from secret KV.
- If UI crashes on public page: do not use `LNbits` or `window.g` unless you explicitly define them.

## Agent Notes (Mirrors `AGENTS.md`)
This extension is a template for WASM-only community extensions. No Python code from the extension runs in LNbits.

What it can do (with permissions):
- Read/write the extension KV store (`ext.db.read_write`).
- Read/write secret KV via `db_secret_get/db_secret_set` (not exposed to public pages).
- Call internal LNbits endpoints via host HTTP (`api.METHOD:/path`).
- Publish websocket events via `ws_publish` (namespace must start with `ext_id:`).
- Register backend payment watchers (`ext.payments.watch`).

What it cannot do:
- Access the filesystem.
- Access environment variables or LNbits settings (outside of API calls).
- Make outbound network calls.
- Execute Python or shell commands.

Files to change:
- `lnbits/extensions/paidtasks/config.json`
- `lnbits/extensions/paidtasks/wasm/`
- `lnbits/extensions/paidtasks/static/` and `lnbits/extensions/paidtasks/templates/`
