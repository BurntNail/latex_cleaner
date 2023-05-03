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
                && self.0.get(ext).map_or(false, |inner_contains| {
                    inner_contains
                        .iter()
                        .all(|inner_contains| string.contains(inner_contains))
                })
        });

        Ok(works)
    }
}
