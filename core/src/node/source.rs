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
    pub fn read_code<P>(filename: P) -> String
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
        // TODO: Recursively include other files
        for p in it {
            buffer += &Source::remove_suffix(&p, "#").replace("\t", "");
            buffer += " ";
        }
        buffer
    }
    /// Saves the lines of the file specified by the filename P
    /// Omits lines that start with # (comments)
    pub fn read_debug_code<P>(filename: P) -> String
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

    /// Strip everything after suffix from string
    fn remove_suffix<'a>(s: &'a str, suffix: &str) -> &'a str {
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
        // TODO: Remove absolute path
        let sp_sources = Source::read_debug_code(String::from(
            "/home/workspace/dhtm_node/src/core/spatial_pooler.push",
        ));
        println!("{}", sp_sources);
        assert!(!sp_sources.contains("#"));
    }
}
