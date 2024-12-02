// You are given an array of distinct integers nums, sorted in ascending order, and an integer target.

// Implement a function to search for target within nums. If it exists, then return its index, otherwise, return -1.

// Your solution must run in O(logn)

// Recommended complexity: Time: O(logn) Space: O(1)

// Hint 1: Can you find an algorithm that is useful when the array is sorted? Maybe other than linear seacrh.

// Hint 2: The problem name is the name of the algorithm that we can use. We need to find a target value and if it does not exist in the array return -1. We have l and r as the boundaries of the segment of the array in which we are searching. Try building conditions to eliminate half of the search segment at each step. Maybe sorted nature of the array can be helpful.

// Hint 3: We compare the target value with the mid of the segment. For example, consider the array [1, 2, 3, 4, 5] and target = 4. The mid value is 3, thus, on the next iteration we search to the right of mid. The remaining segment is [4,5]. Why?

// Hint 4: Because the array is sorted, all elements to the left of mid (including 3) are guaranteed to be smaller than the target. Therefore, we can safely eliminate that half of the array from consideration, narrowing the search to the right half and repeat this search until we find the target.

// Time: O(logn) Space: O(logn)
fn search_recursive(nums: &[i32], target: &i32) -> isize {
    return search_recursive_inner(0, nums.len(), nums, target);
}

fn search_recursive_inner(left: usize, right: usize, nums: &[i32], target: &i32) -> isize {
    if left > right {
        return -1;
    }
    let mid = left + (right - left) / 2;
    let guess = nums[mid];
    if &guess == target {
        return mid as isize;
    }
    if &guess < target {
        return search_recursive_inner(mid + 1, right, nums, target);
    } else {
        return search_recursive_inner(0, mid - 1, nums, target);
    }
}

// Time: O(logn) Space: O(1)
fn search_iterative(nums: &[i32], target: &i32) -> isize {
    let (mut left, mut right) = (0, nums.len() - 1);

    while left <= right {
        let mid = left + ((right - left) / 2);

        if &nums[mid] > target {
            right = mid - 1;
        } else if &nums[mid] < target {
            left = mid + 1;
        } else {
            return mid as isize;
        }
    }
    return -1;
}
// Time: O(logn) Space: O(1)
pub fn search_upper_bound(nums: &[i32], target: &i32) -> isize {
    let (mut left, mut right) = (0, nums.len() - 1);

    while left < right {
        let mid = left + ((right - left) / 2);

        if &nums[mid] > target {
            right = mid;
        } else if &nums[mid] <= target {
            left = mid + 1;
        }
    }
    if left > 0 && &nums[left - 1] == target {
        return (left - 1) as isize;
    } else {
        return -1;
    }
}

// Time: O(logn) Space: O(1)
pub fn search_lower_bound(nums: &[i32], target: &i32) -> isize {
    let (mut left, mut right) = (0, nums.len() - 1);

    while left < right {
        let mid = left + ((right - left) / 2);

        if &nums[mid] >= target {
            right = mid;
        } else {
            left = mid + 1;
        }
    }
    if left < nums.len() && &nums[left] == target {
        return left as isize;
    } else {
        return -1;
    }
}

// Time: O(logn) Space: O(1)
pub fn search_built_in(nums: &[i32], target: &i32) -> isize {
    match nums.binary_search(target) {
        Ok(mid) => return mid as isize,
        Err(_) => return -1,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let nums = [-1, 0, 2, 4, 6, 8];
        let target = 4;
        assert_eq!(search_recursive(&nums, &target), 3);
        assert_eq!(search_iterative(&nums, &target), 3);
        assert_eq!(search_upper_bound(&nums, &target), 3);
        assert_eq!(search_lower_bound(&nums, &target), 3);
        assert_eq!(search_built_in(&nums, &target), 3);
    }

    #[test]
    fn example_2() {
        let nums = [-1, 0, 2, 4, 6, 8];
        let target = 3;
        assert_eq!(search_recursive(&nums, &target), -1);
        assert_eq!(search_iterative(&nums, &target), -1);
        assert_eq!(search_upper_bound(&nums, &target), -1);
        assert_eq!(search_lower_bound(&nums, &target), -1);
        assert_eq!(search_built_in(&nums, &target), -1);
    }
}
