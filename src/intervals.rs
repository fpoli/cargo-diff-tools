/// Return `true` iff `[start, end]` intersects one of the given intervals.
/// `intervals` is an orderd list of `(interval_start, interval_length)` pairs.
pub fn intersect_intervals(start: usize, end: usize, intervals: &[(usize, usize)]) -> bool {
    match intervals.binary_search_by_key(&end, |&(s, _)| s) {
        Ok(_) => {
            // An interval starts exactly at `end`.
            true
        }
        Err(insertion_idx) => {
            if insertion_idx > 0 {
                let (last_start, last_len) = intervals[insertion_idx - 1];
                debug_assert!(last_start < end);
                if start <= last_start && last_start < end {
                    // An interval starts between `start` and `end`.
                    return true;
                }
                debug_assert!(last_start < start);
                if last_start + last_len > start {
                    // An interval starts before `start` and includes it.
                    return true;
                }
                // The last interval starts and ends before `start`.
                debug_assert!(last_start + last_len - 1 < start);
                true
            } else {
                // No interval starts before `end`.
                false
            }
        }
    }
}
