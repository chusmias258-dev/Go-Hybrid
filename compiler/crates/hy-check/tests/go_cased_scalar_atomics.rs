use hy_check::{Verdict, check_file};
use hy_syntax::parse_file;

#[test]
fn go_cased_scalar_atomics_pass_spawn_and_chan_boundaries() {
    let src = "\
import \"sync/atomic\"
fn bump_all(b *atomic.Bool, i32p *atomic.Int32, i64p *atomic.Int64, u32p *atomic.Uint32, u64p *atomic.Uint64, up *atomic.Uintptr, out chan<- i64) {
    b.store(true)
    i32p.add(1)
    i64p.add(2)
    u32p.add(3)
    u64p.add(4)
    up.store(5)
    out <- i64p.load()
}
fn main() {
    var b atomic.Bool
    var i32v atomic.Int32
    var i64v atomic.Int64
    var u32v atomic.Uint32
    var u64v atomic.Uint64
    var upv atomic.Uintptr
    out := make(chan i64, 1)
    ptr_ch := make(chan *atomic.Int64, 1)
    ptr_ch <- &i64v
    task bump_all(&b, &i32v, &i64v, &u32v, &u64v, &upv, out)
}
";
    let r = check_file(&parse_file("test.hy", src).expect("parse"));
    assert!(
        r.verdict == Verdict::Pass && r.diagnostics.is_empty(),
        "expected Go-cased scalar atomics to pass Class-2 sendability, got: {:?}",
        r.diagnostics
    );
}

#[test]
fn go_cased_pointer_and_value_atomics_stay_fail_closed() {
    for src in [
        "import \"sync/atomic\"\nfn w(p *atomic.Pointer[int]) { _ = p }\nfn main() { var p atomic.Pointer[int]\n task w(&p) }",
        "import \"sync/atomic\"\nfn w(v *atomic.Value) { _ = v }\nfn main() { var v atomic.Value\n task w(&v) }",
    ] {
        let r = check_file(&parse_file("test.hy", src).expect("parse"));
        assert!(
            r.verdict == Verdict::Reject && r.diagnostics.iter().any(|d| d.code == "HYB-E3005"),
            "expected HYB-E3005 for {src}, got: {:?}",
            r.diagnostics
        );
    }
}
