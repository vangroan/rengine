//! Validation functions for Mod meta data.

use regex::Regex;

pub fn mod_name(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9\-\_]+$").unwrap();

    re.is_match(s)
}
