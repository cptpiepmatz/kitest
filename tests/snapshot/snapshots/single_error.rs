use std::error::Error;

#[cfg_attr(snapshot, test)]
pub fn fail() -> Result<(), Box<dyn Error>> {
    let _ = Err("some error")?;
    Ok(())
}
