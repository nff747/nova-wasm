use nova_wasm::{compile_to_wasm, compile_to_wat};
use std::process::Command;

#[test]
fn test_wasm_magic_header() {
    let source = r#"
        export fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
    "#;

    let wasm = compile_to_wasm(source).expect("Compilation failed");
    assert!(wasm.len() >= 8);
    // Wasm magic bytes: \0asm
    assert_eq!(&wasm[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    // Wasm version: 1
    assert_eq!(&wasm[4..8], &[0x01, 0x00, 0x00, 0x00]);
}

#[test]
fn test_wat_generation() {
    let source = r#"
        export fn sub(a: i32, b: i32) -> i32 {
            return a - b;
        }
    "#;

    let wat = compile_to_wat(source).expect("WAT compilation failed");
    assert!(wat.contains("(module"));
    assert!(wat.contains("(func $sub (export \"sub\") (param $a i32) (param $b i32) (result i32)"));
    assert!(wat.contains("i32.sub"));
}

#[test]
fn test_typechecker_catches_mismatch() {
    let source = r#"
        export fn bad() -> i32 {
            let x: i32 = 3.14;
            return x;
        }
    "#;

    let result = compile_to_wasm(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Type mismatch") || err.contains("mismatch"));
}

#[test]
fn test_typechecker_catches_undefined_variable() {
    let source = r#"
        export fn test() -> i32 {
            return unknown_var + 1;
        }
    "#;

    let result = compile_to_wasm(source);
    assert!(result.is_err());
}

#[test]
fn test_e2e_node_fibonacci() {
    let source = r#"
        export fn fib(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            let mut a: i32 = 0;
            let mut b: i32 = 1;
            let mut i: i32 = 2;
            while i <= n {
                let temp: i32 = a + b;
                a = b;
                b = temp;
                i += 1;
            }
            return b;
        }
    "#;

    let wasm = compile_to_wasm(source).expect("Fib compilation failed");
    
    // Test execution via Node.js WebAssembly runtime
    let tmp_path = std::env::temp_dir().join("nova_test_fib.wasm");
    std::fs::write(&tmp_path, &wasm).expect("Failed to write test wasm");

    let script = format!(
        r#"
        const fs = require('fs');
        const buf = fs.readFileSync('{}');
        WebAssembly.instantiate(buf).then(res => {{
            const fib = res.instance.exports.fib;
            const res10 = fib(10);
            if (res10 !== 55) {{
                console.error("Expected 55, got " + res10);
                process.exit(1);
            }}
            const res0 = fib(0);
            const res1 = fib(1);
            if (res0 !== 0 || res1 !== 1) {{
                process.exit(2);
            }}
            process.exit(0);
        }}).catch(err => {{
            console.error(err);
            process.exit(3);
        }});
        "#,
        tmp_path.display()
    );

    let output = Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("Failed to run Node.js");

    assert!(output.status.success(), "Node.js Wasm verification failed: {}", String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_e2e_node_linear_memory() {
    let source = r#"
        memory 1;

        export fn store_and_accumulate(base_ptr: i32, count: i32) -> i32 {
            let mut i: i32 = 0;
            while i < count {
                let addr: i32 = base_ptr + (i * 4);
                let val: i32 = (i + 1) * 10;
                @store_i32(addr, val);
                i += 1;
            }

            let mut sum: i32 = 0;
            let mut j: i32 = 0;
            while j < count {
                let addr2: i32 = base_ptr + (j * 4);
                sum += @load_i32(addr2);
                j += 1;
            }
            return sum;
        }
    "#;

    let wasm = compile_to_wasm(source).expect("Memory compilation failed");
    let tmp_path = std::env::temp_dir().join("nova_test_mem.wasm");
    std::fs::write(&tmp_path, &wasm).expect("Failed to write test wasm");

    let script = format!(
        r#"
        const fs = require('fs');
        const buf = fs.readFileSync('{}');
        WebAssembly.instantiate(buf).then(res => {{
            const fn = res.instance.exports.store_and_accumulate;
            const result = fn(64, 5);
            if (result !== 150) {{
                console.error("Expected 150, got", result);
                process.exit(1);
            }}
            process.exit(0);
        }}).catch(err => {{
            console.error(err);
            process.exit(2);
        }});
        "#,
        tmp_path.display()
    );

    let output = Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("Failed to run Node.js");

    assert!(output.status.success(), "Memory Wasm verification failed: {}", String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_while_loop_with_break() {
    let source = r#"
        export fn loop_break() -> i32 {
            let mut i: i32 = 0;
            while i < 100 {
                if i == 42 {
                    break;
                }
                i += 1;
            }
            return i;
        }
    "#;

    let wasm = compile_to_wasm(source).expect("Break compilation failed");
    let tmp_path = std::env::temp_dir().join("nova_test_break.wasm");
    std::fs::write(&tmp_path, &wasm).expect("Failed to write test wasm");

    let script = format!(
        r#"
        const fs = require('fs');
        const buf = fs.readFileSync('{}');
        WebAssembly.instantiate(buf).then(res => {{
            const val = res.instance.exports.loop_break();
            if (val !== 42) {{
                console.error("Expected 42, got", val);
                process.exit(1);
            }}
            process.exit(0);
        }}).catch(err => {{
            console.error(err);
            process.exit(2);
        }});
        "#,
        tmp_path.display()
    );

    let output = Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("Failed to run Node.js");

    assert!(output.status.success(), "Break statement Wasm verification failed: {}", String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_host_import_and_execution() {
    let source = r#"
        import "env" "host_add" fn host_add(a: i32, b: i32) -> i32;

        export fn call_host(x: i32) -> i32 {
            return host_add(x, 100);
        }
    "#;

    let wasm = compile_to_wasm(source).expect("Host import compilation failed");
    let tmp_path = std::env::temp_dir().join("nova_test_import.wasm");
    std::fs::write(&tmp_path, &wasm).expect("Failed to write test wasm");

    let script = format!(
        r#"
        const fs = require('fs');
        const buf = fs.readFileSync('{}');
        const importObject = {{
            env: {{
                host_add: (a, b) => a + b
            }}
        }};
        WebAssembly.instantiate(buf, importObject).then(res => {{
            const val = res.instance.exports.call_host(42);
            if (val !== 142) {{
                console.error("Expected 142, got", val);
                process.exit(1);
            }}
            process.exit(0);
        }}).catch(err => {{
            console.error(err);
            process.exit(2);
        }});
        "#,
        tmp_path.display()
    );

    let output = Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("Failed to run Node.js");

    assert!(output.status.success(), "Host import execution failed: {}", String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn test_string_literal_data_section() {
    let source = r#"
        export fn get_greeting() -> i32 {
            let str: i32 = "Hello, Nova Wasm!";
            return str;
        }
    "#;

    let wasm = compile_to_wasm(source).expect("String literal compilation failed");
    let tmp_path = std::env::temp_dir().join("nova_test_string.wasm");
    std::fs::write(&tmp_path, &wasm).expect("Failed to write test wasm");

    let script = format!(
        r#"
        const fs = require('fs');
        const buf = fs.readFileSync('{}');
        WebAssembly.instantiate(buf).then(res => {{
            const ptr = res.instance.exports.get_greeting();
            const mem = new Uint8Array(res.instance.exports.memory.buffer);
            let end = ptr;
            while (mem[end] !== 0) {{
                end++;
            }}
            const str = new TextDecoder('utf-8').decode(mem.subarray(ptr, end));
            if (str !== "Hello, Nova Wasm!") {{
                console.error("Expected 'Hello, Nova Wasm!', got:", str);
                process.exit(1);
            }}
            process.exit(0);
        }}).catch(err => {{
            console.error(err);
            process.exit(2);
        }});
        "#,
        tmp_path.display()
    );

    let output = Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("Failed to run Node.js");

    assert!(output.status.success(), "String literal Data Section verification failed: {}", String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_file(tmp_path);
}

