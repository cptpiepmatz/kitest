#[cfg_attr(snapshot, test)]
#[should_panic]
pub fn no_panic_when_expected() {
    // mishap: no panic happens
    if false {
        panic!("did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic(expected = "did panic")]
pub fn no_panic_with_expected_message() {
    // mishap: no panic, even though a message is expected
    if false {
        panic!("did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic]
pub fn panic_any() {
    if true {
        panic!("did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic(expected = "did panic")]
pub fn panic_from_assert() {
    if true {
        assert!(false, "did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic(expected = "did panic")]
pub fn panic_with_matching_message() {
    if true {
        panic!("did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic(expected = "panic")]
pub fn panic_with_partial_message_match() {
    if true {
        panic!("did panic");
    }
}

#[cfg_attr(snapshot, test)]
#[should_panic(expected = "other")]
pub fn panic_with_wrong_message() {
    // mishap: message does not match
    if true {
        panic!("did panic");
    }
}
