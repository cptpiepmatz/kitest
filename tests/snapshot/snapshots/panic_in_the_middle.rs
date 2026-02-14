#[cfg_attr(snapshot, test)]
pub fn a_ok() {}

#[cfg_attr(snapshot, test)]
pub fn b_panic() {
    if true {
        panic!()
    }
}

#[cfg_attr(snapshot, test)]
pub fn c_ok() {}

#[cfg_attr(snapshot, test)]
pub fn d_panic() {
    if true {
        panic!()
    }
}

#[cfg_attr(snapshot, test)]
pub fn e_panic() {
    if true {
        panic!()
    }
}

#[cfg_attr(snapshot, test)]
pub fn f_ok() {}
