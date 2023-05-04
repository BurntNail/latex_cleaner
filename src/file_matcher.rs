use std::{collections::HashMap, ffi::OsStr, path::Path};
use strum::Display;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, Display)]
pub enum MatchError {
    CannotConvertToString,
    MissingExtension,
    Construction(FileMatcherConstructionError),
}

#[derive(Error, Debug, Copy, Clone, Display)]
pub enum FileMatcherConstructionError {
    NotEnoughElements,
}

pub struct FileMatcher(HashMap<String, Vec<String>>);

impl<'a, 'b, S: ToString> TryFrom<&'a [&'b [S]]> for FileMatcher {
    type Error = MatchError;

    fn try_from(value: &'a [&'b [S]]) -> Result<Self, Self::Error> {
        let mut map = HashMap::new();
        for list in value {
            let Some(ext) = list.get(0).map(ToString::to_string) else {
                return Err(MatchError::Construction(FileMatcherConstructionError::NotEnoughElements));
            };
            map.insert(ext, list[1..].iter().map(ToString::to_string).collect());
        }

        Ok(Self(map))
    }
}

impl FileMatcher {
    pub fn matches(&self, candidate: &Path) -> Result<bool, MatchError> {
        let extension = match candidate.extension().map(OsStr::to_str) {
            None => return Err(MatchError::MissingExtension),
            Some(None) => return Err(MatchError::CannotConvertToString),
            Some(Some(x)) => x,
        };
        let Some(string) = candidate.to_str() else {
            return Err(MatchError::CannotConvertToString);
        };

        let works = self.0.keys().any(|ext| {
            extension == ext
                && self.0.get(ext).map_or(true, |inner_contains| {
                    inner_contains
                        .iter()
                        .all(|inner_contains| string.contains(inner_contains))
                })
        });

        Ok(works)
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::file_matcher::MatchError;

    use super::FileMatcher;

    pub fn testing_matcher () -> FileMatcher {
        FileMatcher::try_from(
            [
                ["aux"].as_slice(),
                ["log"].as_slice(),
                ["gz", "synctex"].as_slice(),
                ["foo", "bar", "baz"].as_slice()
            ]
            .as_slice(), 
        ).unwrap()
    }

    #[test]
    pub fn working_file_matcher () {
        testing_matcher();
    }

    #[test]
    pub fn empty_file_matcher () {
        let interior: [&[String]; 0] = [];

        FileMatcher::try_from(
            interior.as_slice(), 
        ).unwrap();
    }

    #[test]
    #[should_panic]
    pub fn empty_file_matcher_interior () {
        FileMatcher::try_from(
            [
                ["works"].as_slice(),
                [].as_slice()
            ].as_slice()
        ).unwrap();
    }

    #[test]
    pub fn empty_test_matches () {
        let m = testing_matcher();

        assert!(matches!(m.matches(&PathBuf::from("")), Err(MatchError::MissingExtension)));
    }

    #[test]
    pub fn match_failures () {
        let m = testing_matcher();

        assert!(matches!(m.matches(&PathBuf::from(".")), Err(MatchError::MissingExtension)));
        assert!(matches!(m.matches(&PathBuf::from("abc.")), Ok(false)));
    }

    #[test]
    pub fn only_extension_matches () {
        let m = testing_matcher();

        assert!(matches!(m.matches(&PathBuf::from("x.aux")), Ok(true)));
        assert!(matches!(m.matches(&PathBuf::from(".aux")), Err(MatchError::MissingExtension)));
        assert!(matches!(m.matches(&PathBuf::from("aux.")), Ok(false)));
        assert!(matches!(m.matches(&PathBuf::from("aux")), Err(MatchError::MissingExtension)));
        assert!(matches!(m.matches(&PathBuf::from("y.abcdef.aux")), Ok(true)));
        assert!(matches!(m.matches(&PathBuf::from("bar.txt")), Ok(false)));
    }

    #[test]
    pub fn extension_and_inner_matches () {
        let m = testing_matcher();

        assert!(matches!(m.matches(&PathBuf::from("a.synctex")), Ok(false)));
        assert!(matches!(m.matches(&PathBuf::from("a.synctex.gz")), Ok(true)));
        assert!(matches!(m.matches(&PathBuf::from("bar baz.foo")), Ok(true)));
        assert!(matches!(m.matches(&PathBuf::from("baz bar.foo")), Ok(true)));
        assert!(matches!(m.matches(&PathBuf::from("bar.foo")), Ok(false)));
        assert!(matches!(m.matches(&PathBuf::from("foo.bar")), Ok(false)));
        assert!(matches!(m.matches(&PathBuf::from(".foo")), Err(MatchError::MissingExtension)));
        assert!(matches!(m.matches(&PathBuf::from("important backup.tar.gz")), Ok(false)));
    }
}