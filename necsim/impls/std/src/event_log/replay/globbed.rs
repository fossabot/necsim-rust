use std::path::PathBuf;

use glob::MatchOptions;
use serde::{Deserialize, Deserializer};

use super::segment::SortedSegment;

#[allow(clippy::module_name_repetitions)]
pub struct GlobbedSortedSegments {
    segments: Vec<SortedSegment>,
}

impl<'de> Deserialize<'de> for GlobbedSortedSegments {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let pattern = String::deserialize(deserializer)?;

        let mut paths = Vec::new();

        for path in glob::glob_with(
            &pattern,
            MatchOptions {
                case_sensitive: true,
                require_literal_separator: true,
                require_literal_leading_dot: false,
            },
        )
        .map_err(serde::de::Error::custom)?
        {
            paths.push(path.map_err(serde::de::Error::custom)?);
        }

        let mut segments = Vec::with_capacity(paths.len().min(1));

        let capacity: usize = 100_000;

        if paths.is_empty() {
            segments.push(
                SortedSegment::try_new(&PathBuf::from(pattern), capacity)
                    .map_err(serde::de::Error::custom)?,
            );
        } else if paths.len() == 1 {
            segments.push(
                SortedSegment::try_new(&paths[0], capacity).map_err(serde::de::Error::custom)?,
            );
        } else {
            for path in paths {
                if path.is_file() || !path.exists() {
                    segments.push(
                        SortedSegment::try_new(&path, capacity)
                            .map_err(serde::de::Error::custom)?,
                    );
                }
            }
        }

        Ok(Self { segments })
    }
}

impl IntoIterator for GlobbedSortedSegments {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = SortedSegment;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}
