#[cfg_attr(snapshot, test)]
pub fn a() {
    if true {
        panic!("a");
    }
}

#[cfg_attr(snapshot, test)]
pub fn b() {
    if true {
        panic!("b");
    }
}

#[cfg_attr(snapshot, test)]
pub fn c() {
    if true {
        panic!("c");
    }
}

#[cfg_attr(snapshot, test)]
pub fn d() {
    if true {
        panic!("d");
    }
}