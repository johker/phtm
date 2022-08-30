// Replicator Module
// Reads source code of current node
// Writes transpiled source code to new node.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Source {
    pub filenames: String,
}

impl Source {

    /// Saves the lines of the file specified by the filename P
    /// Omits lines that start with # (comments) and BP (breakpoints)
    pub fn read_code_file<P>(filename: P) -> String
    where
        P: AsRef<Path>,
    {
        let mut buffer: String = String::new();
        let f = BufReader::new(File::open(filename).unwrap());
        let it = f
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| !line.starts_with("#"))
            .filter(|line| !line.starts_with("BP"));
        for p in it {
            buffer += &Source::remove_suffix(&p, "#").replace("\t", "");
            buffer += " ";
        }
        buffer
    }

    /// Saves the lines of the file specified by the filename P
    /// Omits lines that start with # (comments)
    pub fn read_debug_code_file<P>(filename: P) -> String
    where
        P: AsRef<Path>,
    {
        let mut buffer: String = String::new();
        let f = BufReader::new(File::open(filename).unwrap());
        let it = f
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| !line.starts_with("#"));
        for p in it {
            buffer += &Source::remove_suffix(&p, "#").replace("\t", "");
            buffer += " ";
        }
        buffer
    }

    /// Removes lines that start with # (comments) and BP (breakpoints).
    pub fn read_code(s: String) -> String 
    {
        let mut buffer: String = String::new();
        let it = s
            .lines()
            .filter(|line| !line.starts_with("#"))
            .filter(|line| !line.starts_with("BP"));
        for p in it {
            buffer += &Source::remove_suffix(&p, "#").replace("\t", "");
            buffer += " ";
        }
        buffer
    }

    /// Removes lines that start with # (comments)
    pub fn read_debug_code(s: String) -> String 
    {
        let mut buffer: String = String::new();
        let it = s
            .lines()
            .filter(|line| !line.starts_with("#"));
        for p in it {
            buffer += &Source::remove_suffix(&p, "#").replace("\t", "");
            buffer += " ";
        }
        buffer

    }

    /// Strip everything after suffix from string
    pub fn remove_suffix<'a>(s: &'a str, suffix: &str) -> &'a str {
        match s.split(suffix).nth(0) {
            Some(s) => s,
            None => s,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn read_code_removes_comments() {
        let sp_code = include_str!("../core/spatial_pooler.push").to_string();
        let sp_sources = Source::read_debug_code(sp_code);
        assert!(!sp_sources.contains("#"));
    }
}
