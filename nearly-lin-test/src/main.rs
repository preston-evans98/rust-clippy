use nearly_linear::{DropGuard, NearlyLinear};

fn test_bad2() {
    let _a = DropGuard::new(1u8);
}

fn test_bad() {
    let mut a = DropGuard::new(Vec::new());
    a.push(33232);
}

fn test_good() {
    let mut a = DropGuard::new(Vec::new());
    a.push(33232);
    a.done();
}

fn main() {
    test_good();
    test_bad();
    test_bad2()
}
