#[cfg_attr(snapshot, test)]
pub fn panic() {
    if true {
        panic!("some message");
    }
}