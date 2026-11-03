use hy_emit::emit_go;
use hy_syntax::parse_file;
use std::process::Command;

#[test]
fn go_cased_atomics_and_stdlib_shapes_emit_and_run() {
    let src = "\
package main
import \"fmt\"
import \"net/http\"
import \"net/url\"
import \"sync/atomic\"
import \"time\"

struct worker_counters {
    ready atomic.Bool
    c32 atomic.Int32
    c64 atomic.Int64
    u32c atomic.Uint32
    u64c atomic.Uint64
    up atomic.Uintptr
}

fn read_query(r *http.Request) string {
    return r.url.query.get(\"k\")
}

fn make_echo() http.Handler {
    return fn(w, r) {
        w.header.set(\"X-Key\", r.url.query.get(\"k\"))
        w.write_header(204)
    }
}

fn dur_ms(d time.Duration) i64 {
    return d.milliseconds
}

fn main() {
    s := worker_counters{}
    s.ready.store(true)
    ok := s.ready.compare_and_swap(true, false)
    s.c32.add(3)
    s.c64.add(7)
    s.u32c.add(5)
    s.u64c.swap(9)
    s.up.store(11)

    var u url.URL
    u.raw_query = \"k=ok\"
    var req http.Request
    req.url = &u

    var mux http.ServeMux
    mux.handle(\"/echo\", make_echo())

    ms := dur_ms(15 * time.millisecond)
    fmt.println(ok, s.c32.load(), s.c64.load(), s.u32c.load(), s.u64c.load(), s.up.load(), read_query(&req), ms)
}
";
    let go_src = emit_go(&parse_file("test.hy", src).expect("parse")).expect("emit");
    for expected in [
        "s.ready.CompareAndSwap(true, false)",
        "s.c64.Add(7)",
        "r.URL.Query().Get(\"k\")",
        "http.HandlerFunc(",
        "w.Header().Set(",
        "mux.Handle(\"/echo\"",
        "d.Milliseconds()",
    ] {
        assert!(go_src.contains(expected), "missing `{expected}` in:\n{go_src}");
    }
    let go_file = std::env::temp_dir().join("hy_go_cased_shapes_and_atomics.go");
    std::fs::write(&go_file, &go_src).expect("write");
    let out = Command::new("go")
        .args(["run", go_file.to_str().unwrap()])
        .output()
        .expect("go run");
    let _ = std::fs::remove_file(&go_file);
    assert!(
        out.status.success(),
        "{}\n---\n{}",
        String::from_utf8_lossy(&out.stderr),
        go_src
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "true 3 7 5 9 11 ok 15"
    );
}
