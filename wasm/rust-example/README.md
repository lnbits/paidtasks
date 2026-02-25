# Rust example (PaidTasks)

Exports:
- `public_create_invoice(request_id)`
- `noop()`

Uses:
- `db_get` / `db_set`
- `db_secret_get`
- `http_request`

Build:

```bash
cd lnbits/extensions/paidtasks/wasm/rust-example
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/paidtasks.wasm ../module.wasm
```
