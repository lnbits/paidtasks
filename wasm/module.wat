(module
  (import "host" "db_get" (func $db_get (param i32 i32 i32 i32) (result i32)))
  (import "host" "db_set" (func $db_set (param i32 i32 i32 i32) (result i32)))
  (import "host" "db_secret_get" (func $db_secret_get (param i32 i32 i32 i32) (result i32)))
  (import "host" "http_request" (func $http_request (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
  (import "host" "ws_publish" (func $ws_publish (param i32 i32 i32 i32) (result i32)))

  (memory (export "memory") 1)

  (data (i32.const 0) "public_request")
  (data (i32.const 32) "public_response")
  (data (i32.const 64) "task_cost:")
  (data (i32.const 80) "task_list:")
  (data (i32.const 96) "list_wallet_inkey:")
  (data (i32.const 128) "task_paid:")
  (data (i32.const 144) "paidtasks:")
  (data (i32.const 176) "POST")
  (data (i32.const 192) "/api/v1/payments")
  (data (i32.const 256) "{\"out\":false,\"amount\":")
  (data (i32.const 320) ",\"unit\":\"sat\",\"memo\":\"Paid task\"}")
  (data (i32.const 392) "{\"paid\":true}")
  (data (i32.const 424) "{\"paid\":false}")

  (func $memcpy (param $dst i32) (param $src i32) (param $len i32)
    local.get $dst
    local.get $src
    local.get $len
    memory.copy)

  (func $write_key_buf (param $prefix_ptr i32) (param $prefix_len i32) (param $suffix_ptr i32) (param $suffix_len i32) (param $out_ptr i32) (result i32)
    local.get $out_ptr
    local.get $prefix_ptr
    local.get $prefix_len
    call $memcpy
    local.get $out_ptr
    local.get $prefix_len
    i32.add
    local.get $suffix_ptr
    local.get $suffix_len
    call $memcpy
    local.get $prefix_len
    local.get $suffix_len
    i32.add)

  (func $notify_paid (param $request_id i32) (result i32)
    (local $task_len i32)
    (local $topic_len i32)

    ;; db_get public_request -> task id string
    i32.const 0
    i32.const 14
    i32.const 600
    i32.const 64
    call $db_get
    local.set $task_len

    ;; build topic paidtasks:task_paid:<task_id>
    i32.const 144
    i32.const 10
    i32.const 128
    i32.const 10
    i32.const 800
    call $write_key_buf
    drop

    i32.const 800
    i32.const 20
    i32.add
    i32.const 600
    local.get $task_len
    call $memcpy

    i32.const 20
    local.get $task_len
    i32.add
    local.set $topic_len

    ;; ws_publish(topic, payload)
    i32.const 800
    local.get $topic_len
    i32.const 392
    i32.const 14
    call $ws_publish
    drop

    i32.const 0)

  (func $public_task_status (param $request_id i32) (result i32)
    (local $task_len i32)
    (local $paid_len i32)

    ;; db_get public_request -> task id string
    i32.const 0
    i32.const 14
    i32.const 600
    i32.const 64
    call $db_get
    local.set $task_len

    ;; build task_paid:<task_id>
    i32.const 128
    i32.const 10
    i32.const 600
    local.get $task_len
    i32.const 400
    call $write_key_buf
    drop

    ;; db_get paid
    i32.const 400
    i32.const 10
    local.get $task_len
    i32.add
    i32.const 700
    i32.const 32
    call $db_get
    local.set $paid_len

    ;; store response in public_response
    i32.const 32
    i32.const 15
    i32.const 700
    local.get $paid_len
    i32.eqz
    if
      i32.const 424
      i32.const 16
    else
      i32.const 392
      i32.const 14
    end
    call $db_set
    drop

    i32.const 0)

  (func $public_create_invoice (param $request_id i32) (result i32)
    (local $task_len i32)
    (local $cost_len i32)
    (local $list_len i32)
    (local $inkey_len i32)
    (local $body_len i32)
    (local $resp_len i32)

    ;; db_get public_request -> task id string
    i32.const 0
    i32.const 14
    i32.const 600
    i32.const 64
    call $db_get
    local.set $task_len

    ;; build task_cost:<task_id>
    i32.const 64
    i32.const 10
    i32.const 600
    local.get $task_len
    i32.const 400
    call $write_key_buf
    drop

    ;; db_get cost
    i32.const 400
    i32.const 10
    local.get $task_len
    i32.add
    i32.const 700
    i32.const 32
    call $db_get
    local.set $cost_len

    ;; build task_list:<task_id>
    i32.const 80
    i32.const 10
    i32.const 600
    local.get $task_len
    i32.const 400
    call $write_key_buf
    drop

    ;; db_get list id
    i32.const 400
    i32.const 10
    local.get $task_len
    i32.add
    i32.const 740
    i32.const 32
    call $db_get
    local.set $list_len

    ;; build list_wallet_inkey:<list_id>
    i32.const 96
    i32.const 18
    i32.const 740
    local.get $list_len
    i32.const 400
    call $write_key_buf
    drop

    ;; db_secret_get list wallet inkey
    i32.const 400
    i32.const 18
    local.get $list_len
    i32.add
    i32.const 800
    i32.const 96
    call $db_secret_get
    local.set $inkey_len

    ;; build body prefix + cost + suffix
    i32.const 1000
    i32.const 256
    i32.const 22
    call $memcpy

    i32.const 1000
    i32.const 22
    i32.add
    i32.const 700
    local.get $cost_len
    call $memcpy

    i32.const 1000
    i32.const 22
    i32.add
    local.get $cost_len
    i32.add
    i32.const 320
    i32.const 33
    call $memcpy

    i32.const 22
    local.get $cost_len
    i32.add
    i32.const 33
    i32.add
    local.set $body_len

    ;; http_request
    i32.const 176
    i32.const 4
    i32.const 192
    i32.const 16
    i32.const 1000
    local.get $body_len
    i32.const 800
    local.get $inkey_len
    i32.const 1400
    i32.const 4096
    call $http_request
    local.set $resp_len

    ;; store response in public_response
    i32.const 32
    i32.const 15
    i32.const 1400
    local.get $resp_len
    call $db_set
    drop

    i32.const 0)

  (func $noop (result i32)
    i32.const 0)

  (export "public_create_invoice" (func $public_create_invoice))
  (export "public_task_status" (func $public_task_status))
  (export "notify_paid" (func $notify_paid))
  (export "noop" (func $noop))
)
