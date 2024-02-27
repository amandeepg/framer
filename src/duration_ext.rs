use anyhow::{Context, Result};
use std::time::Duration;

pub trait DurationExt {
    fn display(self) -> Result<String>;
}

impl DurationExt for Duration {
    fn display(self) -> Result<String> {
        let secs = self.as_secs();
        let ms = self.subsec_millis();
        if secs < 1 {
            Ok(format!("{}ms", ms))
        } else {
            Ok(format!(
                "{}.{}s",
                secs,
                ms.to_string().chars().next().context("No first char")?
            ))
        }
    }
}
