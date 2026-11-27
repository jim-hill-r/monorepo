// You should aim for a solution with O(n) time and O(n) space, where n is the size of the input array.

// Hint 1: A brute force solution would be to check every element against every other element in the array. This would be an O(n^2) solution. Can you think of a better way?

// Hint 2: Is there a way to check if an element is a duplicate without comparing it to every other element? Maybe there's a data structure that is useful here.

// Hint 3: We can use a hash data structure like a hash set or hash map to store elements we've already seen. This will allow us to check if an element is a duplicate in constant time.

use std::collections::HashSet;

// Time: O(n^2) Space: O(1)
pub fn has_duplicate_brute_force(nums: &[i64]) -> bool {
    for (i, a) in nums.iter().enumerate() {
        if nums[i + 1..].contains(a) {
            return true;
        }
    }
    false
}

// Time: O(nlogn) Space: O(1) or O(n)
pub fn has_duplicate_sorting(nums: &[i64]) -> bool {
    let mut sorted_nums = nums.to_vec();
    sorted_nums.sort();
    sorted_nums.windows(2).any(|pair| pair[0] == pair[1])
}

// Time: O(n) Space: O(n)
pub fn has_duplicate_hash_set(nums: &[i64]) -> bool {
    let mut seen: HashSet<i64> = HashSet::new();
    for n in nums {
        let is_newly_inserted = seen.insert(*n);
        if !is_newly_inserted {
            return true;
        }
    }

    false
}

// Time: O(n) Space: O(n)
pub fn has_duplicate_hash_set_length(nums: &[i64]) -> bool {
    let set: HashSet<i64> = nums.iter().copied().collect();
    set.len() < nums.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let nums = vec![1, 2, 3, 3];
        assert!(has_duplicate_brute_force(&nums));
        assert!(has_duplicate_sorting(&nums));
        assert!(has_duplicate_hash_set(&nums));
        assert!(has_duplicate_hash_set_length(&nums));
    }

    #[test]
    fn example_2() {
        let nums = vec![1, 2, 3, 4];
        assert!(!has_duplicate_brute_force(&nums));
        assert!(!has_duplicate_sorting(&nums));
        assert!(!has_duplicate_hash_set(&nums));
        assert!(!has_duplicate_hash_set_length(&nums));
    }

    #[test]
    fn empty_and_single() {
        for nums in [vec![], vec![7]] {
            assert!(!has_duplicate_brute_force(&nums));
            assert!(!has_duplicate_sorting(&nums));
            assert!(!has_duplicate_hash_set(&nums));
            assert!(!has_duplicate_hash_set_length(&nums));
        }
    }

    #[test]
    fn duplicates_far_apart() {
        let nums = vec![5, 1, 2, 3, 4, 5];
        assert!(has_duplicate_brute_force(&nums));
        assert!(has_duplicate_sorting(&nums));
        assert!(has_duplicate_hash_set(&nums));
        assert!(has_duplicate_hash_set_length(&nums));
    }
}
