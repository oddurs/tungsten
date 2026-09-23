//! Did-you-mean suggestions for unknown words.

use crate::resolve::vocabulary;
use tungsten_units::all_forms;

/// Optimal string alignment distance (Damerau–Levenshtein with adjacent
/// transpositions), over chars.
pub fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[n][m]
}

/// The closest known word within distance 2 (1 for words of four letters or
/// fewer), or nothing.
pub fn suggest(word: &str) -> Option<String> {
    let lower = word.to_lowercase();
    let len = lower.chars().count();
    let limit = if len <= 4 { 1 } else { 2 };
    let mut best: Option<(usize, bool, usize, &str)> = None;
    for cand in all_forms().chain(vocabulary()) {
        let clen = cand.chars().count();
        if clen.abs_diff(len) > limit {
            continue;
        }
        // Case-insensitive, so KM finds km.
        let d = distance(&lower, &cand.to_lowercase());
        if d > limit {
            continue;
        }
        let same_first =
            cand.chars().next().map(|c| c.to_ascii_lowercase()) == lower.chars().next();
        let key = (d, !same_first, clen.abs_diff(len), cand);
        if best.is_none_or(|b| key < b) {
            best = Some(key);
        }
    }
    best.map(|(_, _, _, c)| c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances() {
        assert_eq!(distance("milez", "miles"), 1);
        assert_eq!(distance("mtere", "metre"), 1);
        assert_eq!(distance("abc", "abc"), 0);
    }

    #[test]
    fn suggestions() {
        assert_eq!(suggest("milez").as_deref(), Some("miles"));
        assert_eq!(suggest("kilometrs").as_deref(), Some("kilometre"));
        assert_eq!(suggest("KM").as_deref(), Some("km"));
        assert_eq!(suggest("zzzzzz"), None);
    }
}
