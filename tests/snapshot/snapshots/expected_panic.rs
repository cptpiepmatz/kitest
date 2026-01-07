#[cfg_attr(snapshot, test)]
#[should_panic]
pub fn panic() {
    if true {
        panic!("did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic]
pub fn no_panic() {
    if false {
        panic!("no panic");
    }
}
