use std::{fmt::{self, Display, Formatter}, error::Error};

#[allow(unused)]
#[derive(Debug)]
struct StructuredError {
    a: String,
    b: u32,
    c: Vec<u8>,
}

impl Display for StructuredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "hey, got an error")
    }
}

impl Error for StructuredError {}

#[cfg_attr(snapshot, test)]
pub fn fail() -> Result<(), Box<dyn Error>> {
    let _ = Err(StructuredError {
        a: String::from("abc"),
        b: 42,
        c: vec![1, 2, 3],
    })?;
    Ok(())
}
