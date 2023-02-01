use crate::path_part::PathPart;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::DirEntry;

/// Find the closest match(es) to the given program name as suggestsions
///
/// Reads in all executables on the PATH and runs a string distance
/// calculation between them and the `program`.
///
/// The top `guess_limit` results will be returned.
///
/// If no results are found, or `guess_limit` is zero then
/// None will be returned.
pub(crate) fn spelling(
    program: &OsString,
    parts: &[PathPart],
    guess_limit: usize,
) -> Option<Vec<OsString>> {
    if guess_limit == 0 {
        return None;
    }

    let mut heap = std::collections::BinaryHeap::new();
    let values = parts
        .par_iter()
        .filter_map(|p| std::fs::read_dir(&p.absolute).ok())
        .flat_map(|r| {
            r.filter_map(std::result::Result::ok)
                .collect::<Vec<DirEntry>>()
        })
        .map(|d| d.path())
        .filter_map(|p| p.file_name().map(std::ffi::OsStr::to_os_string))
        .map(|filename| {
            let score = strsim::normalized_levenshtein(
                &program.to_string_lossy(),
                &filename.to_string_lossy(),
            );

            (ordered_float::OrderedFloat(score), filename)
        })
        .collect::<Vec<(_, _)>>();

    for value in &values {
        heap.push(value);
    }

    if heap.is_empty() {
        None
    } else {
        let mut out = HashSet::new();
        while let Some((_, filename)) = heap.pop() {
            if out.len() >= guess_limit {
                break;
            }
            if filename != program {
                out.insert(filename.clone());
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(out.into_iter().collect_vec())
        }
    }
}
