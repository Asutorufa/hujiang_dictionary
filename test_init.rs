use std::sync::OnceLock;
fn main() {
    let lock = OnceLock::new();
    let _ = lock.get_or_try_init(|| -> Result<i32, ()> { Ok(42) });
}
