#[cfg_attr(snapshot, test)]
pub fn ok_before() {}

#[cfg_attr(snapshot, test)]
pub fn panic_in_the_middle_a() {
    if true {
        panic!()
    }
}

#[cfg_attr(snapshot, test)]
pub fn panic_in_the_middle_b() {
    if true {
        panic!()
    }
}

#[cfg_attr(snapshot, test)]
pub fn ok_after() {}
