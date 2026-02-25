# PaidTasks (WASM)

PaidTasks is a WASM-only extension that lets users create paid task lists with a public page per list.

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

Use `docs/devs/agents_wasm_extensions.md` for AI/agent instructions.

## Test Checklist

1. Enable extension and grant permissions + `paidtasks` tag.
2. Create a list and tasks.
3. Open the public page and create an invoice.
4. Pay the invoice and verify:
   - Task is marked paid.
   - Public page updates via websocket.
   - KV `task_paid:<id>` is set.
