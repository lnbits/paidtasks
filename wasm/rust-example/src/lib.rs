#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { core::arch::wasm32::unreachable() }
}

#[link(wasm_import_module = "host")]
extern "C" {
    fn db_get(key_ptr: *const u8, key_len: i32, out_ptr: *mut u8, out_len: i32) -> i32;
    fn db_set(key_ptr: *const u8, key_len: i32, val_ptr: *const u8, val_len: i32) -> i32;
    fn db_secret_get(key_ptr: *const u8, key_len: i32, out_ptr: *mut u8, out_len: i32) -> i32;
    fn http_request(
        method_ptr: *const u8,
        method_len: i32,
        path_ptr: *const u8,
        path_len: i32,
        body_ptr: *const u8,
        body_len: i32,
        key_ptr: *const u8,
        key_len: i32,
        out_ptr: *mut u8,
        out_len: i32,
    ) -> i32;
}

const KEY_PUBLIC_REQUEST: &[u8] = b"public_request";
const KEY_PUBLIC_RESPONSE: &[u8] = b"public_response";
const KEY_ID_COUNTER: &[u8] = b"id_counter";
const PREFIX_TASK_COST: &[u8] = b"task_cost:";
const PREFIX_TASK_LIST: &[u8] = b"task_list:";
const PREFIX_LIST_WALLET: &[u8] = b"list_wallet_inkey:";

const HTTP_METHOD: &[u8] = b"POST";
const HTTP_PATH: &[u8] = b"/api/v1/payments";
const BODY_PREFIX: &[u8] = b"{\"out\":false,\"amount\":";
const BODY_MID: &[u8] = b",\"unit\":\"sat\",\"memo\":\"Paid task\",\"extra\":{\"tag\":\"paidtasks\"}}";
const ERROR_INVALID_TASK: &[u8] = b"{\"error\":\"Invalid task\"}";
const ID_PREFIX: &[u8] = b"{\"id\":\"pt_";
const ID_SUFFIX: &[u8] = b"\"}";

#[no_mangle]
pub extern "C" fn public_create_invoice(_request_id: i32) -> i32 {
    let (task_id, task_len) = read_public_request();
    if task_len <= 0 {
        let _ = write_response(ERROR_INVALID_TASK);
        return 0;
    }

    let (cost, cost_len) = read_key_bytes(PREFIX_TASK_COST, task_id, task_len, 32);
    let (list_id, list_len) = read_key_bytes(PREFIX_TASK_LIST, task_id, task_len, 64);
    let (inkey, inkey_len) = read_secret_bytes(PREFIX_LIST_WALLET, list_id, list_len, 96);
    if cost_len <= 0 || list_len <= 0 || inkey_len <= 0 {
        let _ = write_response(ERROR_INVALID_TASK);
        return 0;
    }

    let (body, body_len) = build_body(cost, cost_len);

    let mut response = [0u8; 4096];
    let resp_len = unsafe {
        http_request(
            HTTP_METHOD.as_ptr(),
            HTTP_METHOD.len() as i32,
            HTTP_PATH.as_ptr(),
            HTTP_PATH.len() as i32,
            body.as_ptr(),
            body_len,
            inkey.as_ptr(),
            inkey_len,
            response.as_mut_ptr(),
            response.len() as i32,
        )
    };

    let _ = write_response(&response[..resp_len.max(0) as usize]);
    0
}

#[no_mangle]
pub extern "C" fn generate_id(_request_id: i32) -> i32 {
    let (counter_buf, counter_len) =
        read_key_bytes(KEY_ID_COUNTER, [0u8; 64], 0, 32);
    let mut counter = parse_u64(counter_buf, counter_len);
    counter = counter.wrapping_add(1);
    let _ = write_counter(counter);

    let (resp, resp_len) = build_id_response(counter);
    let _ = write_response(&resp[..resp_len as usize]);
    0
}

#[no_mangle]
pub extern "C" fn noop() -> i32 {
    0
}

fn write_response(value: &[u8]) -> bool {
    unsafe {
        db_set(
            KEY_PUBLIC_RESPONSE.as_ptr(),
            KEY_PUBLIC_RESPONSE.len() as i32,
            value.as_ptr(),
            value.len() as i32,
        );
    }
    true
}

fn read_public_request() -> ([u8; 64], i32) {
    let mut buf = [0u8; 64];
    let len = unsafe {
        db_get(
            KEY_PUBLIC_REQUEST.as_ptr(),
            KEY_PUBLIC_REQUEST.len() as i32,
            buf.as_mut_ptr(),
            buf.len() as i32,
        )
    };
    if len <= 0 { ([0u8; 64], 0) } else { (buf, len) }
}

fn read_key_bytes(prefix: &[u8], suffix: [u8; 64], suffix_len: i32, out_len: usize) -> ([u8; 64], i32) {
    let mut key = [0u8; 128];
    let key_len = write_key_buf(&mut key, prefix, &suffix, suffix_len as usize);

    let mut out = [0u8; 64];
    let len = unsafe {
        db_get(
            key.as_ptr(),
            key_len as i32,
            out.as_mut_ptr(),
            out_len as i32,
        )
    };
    if len <= 0 { ([0u8; 64], 0) } else { (out, len) }
}

// task_paid handler removed: paid state is now stored inside the task record.
fn read_secret_bytes(prefix: &[u8], suffix: [u8; 64], suffix_len: i32, out_len: usize) -> ([u8; 96], i32) {
    let mut key = [0u8; 128];
    let key_len = write_key_buf(&mut key, prefix, &suffix, suffix_len as usize);

    let mut out = [0u8; 96];
    let len = unsafe {
        db_secret_get(
            key.as_ptr(),
            key_len as i32,
            out.as_mut_ptr(),
            out_len as i32,
        )
    };
    if len <= 0 { ([0u8; 96], 0) } else { (out, len) }
}

fn build_body(cost: [u8; 64], cost_len: i32) -> ([u8; 256], i32) {
    let mut body = [0u8; 256];
    let mut len = 0usize;
    len += write_bytes(&mut body[len..], BODY_PREFIX);
    len += write_bytes(&mut body[len..], &cost[..cost_len.max(0) as usize]);
    len += write_bytes(&mut body[len..], BODY_MID);
    (body, len as i32)
}

fn write_key_buf(out: &mut [u8], prefix: &[u8], suffix: &[u8; 64], suffix_len: usize) -> usize {
    let mut len = 0usize;
    len += write_bytes(&mut out[len..], prefix);
    if suffix_len > 0 {
        len += write_bytes(&mut out[len..], &suffix[..suffix_len]);
    }
    len
}

fn write_bytes(out: &mut [u8], bytes: &[u8]) -> usize {
    let mut i = 0usize;
    while i < bytes.len() && i < out.len() {
        out[i] = bytes[i];
        i += 1;
    }
    i
}

fn parse_u64(buf: [u8; 64], len: i32) -> u64 {
    if len <= 0 {
        return 0;
    }
    let mut n: u64 = 0;
    let mut i = 0usize;
    while i < len as usize && i < buf.len() {
        let b = buf[i];
        if b < b'0' || b > b'9' {
            break;
        }
        n = n.saturating_mul(10).saturating_add((b - b'0') as u64);
        i += 1;
    }
    n
}

fn write_counter(counter: u64) -> bool {
    let mut buf = [0u8; 32];
    let len = write_u64(&mut buf, counter);
    unsafe {
        db_set(
            KEY_ID_COUNTER.as_ptr(),
            KEY_ID_COUNTER.len() as i32,
            buf.as_ptr(),
            len as i32,
        );
    }
    true
}

fn write_u64(out: &mut [u8], mut value: u64) -> usize {
    if out.is_empty() {
        return 0;
    }
    if value == 0 {
        out[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 20];
    let mut len = 0usize;
    while value > 0 && len < tmp.len() {
        let digit = (value % 10) as u8;
        tmp[len] = b'0' + digit;
        value /= 10;
        len += 1;
    }
    let mut i = 0usize;
    while i < len && i < out.len() {
        out[i] = tmp[len - 1 - i];
        i += 1;
    }
    i
}

fn build_id_response(counter: u64) -> ([u8; 64], i32) {
    let mut buf = [0u8; 64];
    let mut len = 0usize;
    len += write_bytes(&mut buf[len..], ID_PREFIX);
    len += write_u64(&mut buf[len..], counter);
    len += write_bytes(&mut buf[len..], ID_SUFFIX);
    (buf, len as i32)
}
