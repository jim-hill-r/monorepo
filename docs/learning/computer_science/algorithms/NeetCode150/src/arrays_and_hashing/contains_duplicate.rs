// You should aim for a solution with O(n) time and O(n) space, where n is the size of the input array.

// Hint 1: A brute force solution would be to check every element against every other element in the array. This would be an O(n^2) solution. Can you think of a better way?

// Hint 2: Is there a way to check if an element is a duplicate without comparing it to every other element? Maybe there's a data structure that is useful here.

// Hint 3: We can use a hash data structure like a hash set or hash map to store elements we've already seen. This will allow us to check if an element is a duplicate in constant time.

use std::collections::HashSet;

// Time: O(n^2) Space: O(1)
pub fn has_duplicate_brute_force(nums: &Vec<i64>) -> bool {
    let l = nums.len();
    for i in 0..l {
        for j in i + 1..l {
            if nums[i] == nums[j] {
                return true;
            }
        }
    }
    return false;
}

// Time: O(nlogn) Space: O(1) or O(n)
pub fn has_duplicate_sorting(nums: &Vec<i64>) -> bool {
    let mut sorted_nums = nums.clone();
    sorted_nums.sort();
    for i in 1..sorted_nums.len() {
        if sorted_nums[i] == sorted_nums[i - 1] {
            return true;
        }
    }
    return false;
}

// Time: O(n) Space: O(n)
pub fn has_duplicate_hash_set(nums: &Vec<i64>) -> bool {
    let mut seen: HashSet<i64> = HashSet::new();
    for n in nums {
        let is_newly_inserted = seen.insert(*n);
        if !is_newly_inserted {
            return true;
        }
    }

    return false;
}

// Time: O(n) Space: O(n)
pub fn has_duplicate_hash_set_length(nums: &Vec<i64>) -> bool {
    let set: HashSet<i64> = nums.iter().cloned().collect();
    return set.len() < nums.len();
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
        // assert!(!has_duplicate_sorting(&nums));
        // assert!(!has_duplicate_hash_set(&nums));
        // assert!(!has_duplicate_hash_set_length(&nums));
    }
}
