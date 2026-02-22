#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
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
    fn ws_publish(topic_ptr: *const u8, topic_len: i32, data_ptr: *const u8, data_len: i32) -> i32;
}

const KEY_PUBLIC_REQUEST: &[u8] = b"public_request";
const KEY_PUBLIC_RESPONSE: &[u8] = b"public_response";
const PREFIX_TASK_COST: &[u8] = b"task_cost:";
const PREFIX_TASK_LIST: &[u8] = b"task_list:";
const PREFIX_TASK_PAID: &[u8] = b"task_paid:";
const PREFIX_LIST_WALLET: &[u8] = b"list_wallet_inkey:";
const WS_PREFIX: &[u8] = b"paidtasks:";
const WS_SUFFIX: &[u8] = b"task_paid:";

const HTTP_METHOD: &[u8] = b"POST";
const HTTP_PATH: &[u8] = b"/api/v1/payments";
const BODY_PREFIX: &[u8] = b"{\"out\":false,\"amount\":";
const BODY_SUFFIX: &[u8] = b",\"unit\":\"sat\",\"memo\":\"Paid task\"}";
const PAID_TRUE_JSON: &[u8] = b"{\"paid\":true}";
const PAID_FALSE_JSON: &[u8] = b"{\"paid\":false}";

#[no_mangle]
pub extern "C" fn public_create_invoice(_request_id: i32) -> i32 {
    let (task_id, task_len) = read_public_request();
    if task_len <= 0 {
        let _ = write_response(PAID_FALSE_JSON);
        return 0;
    }

    let (cost, cost_len) = read_key_bytes(PREFIX_TASK_COST, task_id, task_len, 32);
    let (list_id, list_len) = read_key_bytes(PREFIX_TASK_LIST, task_id, task_len, 32);
    let (inkey, inkey_len) = read_secret_bytes(PREFIX_LIST_WALLET, list_id, list_len, 96);

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
pub extern "C" fn public_task_status(_request_id: i32) -> i32 {
    let (task_id, task_len) = read_public_request();
    if task_len <= 0 {
        let _ = write_response(PAID_FALSE_JSON);
        return 0;
    }

    let (_paid_buf, paid_len) = read_key_bytes(PREFIX_TASK_PAID, task_id, task_len, 32);
    if paid_len <= 0 {
        let _ = write_response(PAID_FALSE_JSON);
    } else {
        let _ = write_response(PAID_TRUE_JSON);
    }
    0
}

#[no_mangle]
pub extern "C" fn notify_paid(_request_id: i32) -> i32 {
    let (task_id, task_len) = read_public_request();
    if task_len <= 0 {
        return 0;
    }

    let (topic, topic_len) = build_ws_topic(task_id, task_len);
    unsafe {
        ws_publish(
            topic.as_ptr(),
            topic_len,
            PAID_TRUE_JSON.as_ptr(),
            PAID_TRUE_JSON.len() as i32,
        );
    }
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
    len += write_bytes(&mut body[len..], BODY_SUFFIX);
    (body, len as i32)
}

fn build_ws_topic(task_id: [u8; 64], task_len: i32) -> ([u8; 128], i32) {
    let mut buf = [0u8; 128];
    let mut len = 0usize;
    len += write_bytes(&mut buf[len..], WS_PREFIX);
    len += write_bytes(&mut buf[len..], WS_SUFFIX);
    len += write_bytes(&mut buf[len..], &task_id[..task_len.max(0) as usize]);
    (buf, len as i32)
}

fn write_key_buf(out: &mut [u8], prefix: &[u8], suffix: &[u8; 64], suffix_len: usize) -> usize {
    let mut len = 0usize;
    len += write_bytes(&mut out[len..], prefix);
    len += write_bytes(&mut out[len..], &suffix[..suffix_len]);
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
