use std::{collections::HashMap, path::PathBuf};
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
    pub fn matches(&self, candidate: PathBuf) -> Result<bool, MatchError> {
        let extension = match candidate.extension().map(|x| x.to_str()) {
            None => return Err(MatchError::MissingExtension),
            Some(None) => return Err(MatchError::CannotConvertToString),
            Some(Some(x)) => x,
        };
        let Some(string) = candidate.to_str() else {
            return Err(MatchError::CannotConvertToString);
        };

        let mut works = false;

        for ext in self.0.keys() {
            if ext == extension {
                let mut inner_works = true;

                for needs_inner in self.0[ext] {
                    if !string.contains(&needs_inner) {
                        inner_works = false;
                    }
                }

                if inner_works {
                    works = true;
                    break;
                }
            }
        }

        Ok(works)
    }
}
