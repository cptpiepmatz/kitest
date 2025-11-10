#[cfg_attr(snapshot, test)]
#[ignore]
pub fn first() {}

#[cfg_attr(snapshot, test)]
#[ignore]
pub fn second() {}

#[cfg_attr(snapshot, test)]
#[ignore = "reasons"]
pub fn third() {}