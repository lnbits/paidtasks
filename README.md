# PaidTasks (WASM)

This is an example WASM extension you can use as a base to vibe code an extension that can be shared safely.

PaidTasks is an extension to share a list of tasks and have people pay you to complete the tasks.

## Key Files

- `config.json`: permissions, public handlers, public KV keys, and payment tags
- `wasm/`: `module.wat` or `module.wasm`
- `static/` and `templates/`: UI and public pages

## Permissions (Current)

- `ext.db.read_write`
- `api.POST:/api/v1/payments`
- `ext.payments.watch`

## Payment Tags

This extension uses the tag `paidtasks`. Users must grant it in the permissions dialog.

## Agent Guidance

Use `lnbits/extensions/wasm/docs/agents_wasm_extensions.md` for AI/agent instructions.

## Test Checklist

1. Enable extension and grant permissions + `paidtasks` tag.
2. Create a list and tasks.
3. Open the public page and create an invoice.
4. Pay the invoice and verify:
   - Task is marked paid.
   - Public page updates via websocket.
