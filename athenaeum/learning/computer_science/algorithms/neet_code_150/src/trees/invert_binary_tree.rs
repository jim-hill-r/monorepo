// You are given the root of a binary tree root. Invert the binary tree and return its root.

// Recommended complexity: Time: O(n) Space: O(n)

// Hint 1: From the diagram, you can see that the left and right children of every node in the tree are swapped. Can you think of a way to achieve this recursively? Maybe an algorithm that is helpful to traverse the tree.

// Hint 2: We can use the Depth First Search (DFS) algorithm. At each node, we swap its left and right children by swapping their pointers. This inverts the current node, but every node in the tree also needs to be inverted. To achieve this, we recursively visit the left and right children and perform the same operation. If the current node is null, we simply return.

use std::collections::VecDeque;
use std::fmt::Debug;

/// A binary tree node, filled level by level from a list of values.
#[derive(Debug, Clone)]
pub struct Tree<T: Clone + Debug> {
    pub value: T,
    pub left: Option<Box<Tree<T>>>,
    pub right: Option<Box<Tree<T>>>,
}

impl<T: Clone + Debug> Tree<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }

    pub fn from_vec(values: &[T]) -> Option<Tree<T>> {
        let (first, rest) = values.split_first()?;
        let mut root: Tree<T> = Self::new(first.clone());
        for value in rest {
            root.insert(value.clone());
        }
        Some(root)
    }

    // Breadth first traverse
    pub fn to_vec(&self) -> Vec<T> {
        let mut ret: Vec<T> = vec![];
        let mut queue: VecDeque<&Tree<T>> = VecDeque::new();
        queue.push_front(self);

        while let Some(current) = queue.pop_back() {
            ret.push(current.value.clone());
            if let Some(node) = &current.left {
                queue.push_front(node);
            }
            if let Some(node) = &current.right {
                queue.push_front(node);
            }
        }

        ret
    }

    /// Puts the value in the first free child slot, level by level.
    pub fn insert(&mut self, value: T) {
        let mut queue: VecDeque<&mut Tree<T>> = VecDeque::new();
        queue.push_front(self);

        // The queue only empties after a free slot is found, because every
        // tree has one.
        while let Some(current) = queue.pop_back() {
            let Tree { left, right, .. } = current;

            match left {
                Some(node) => {
                    queue.push_front(node);
                }
                None => {
                    *left = Some(Box::new(Tree::new(value)));
                    return;
                }
            }

            match right {
                Some(node) => {
                    queue.push_front(node);
                }
                None => {
                    *right = Some(Box::new(Tree::new(value)));
                    return;
                }
            }
        }
    }
}

// Time: O(n) Space: O(n)
pub fn invert_binary_tree_breadth_first_search<T: Clone + Debug>(
    node: Option<Box<Tree<T>>>,
) -> Option<Box<Tree<T>>> {
    let mut tree = node?;
    let mut queue: VecDeque<&mut Tree<T>> = VecDeque::new();
    queue.push_front(&mut tree);

    while let Some(current) = queue.pop_back() {
        let Tree { left, right, .. } = current;
        std::mem::swap(left, right);

        if let Some(node) = left {
            queue.push_front(node);
        }
        if let Some(node) = right {
            queue.push_front(node);
        }
    }

    Some(tree)
}

// Time: O(n) Space: O(n)
pub fn invert_binary_tree_depth_first_search<T: Clone + Debug>(
    node: Option<Box<Tree<T>>>,
) -> Option<Box<Tree<T>>> {
    let tree = node?;
    let mut new_tree = Tree::new(tree.value);
    new_tree.left = invert_binary_tree_depth_first_search(tree.right);
    new_tree.right = invert_binary_tree_depth_first_search(tree.left);

    Some(Box::new(new_tree))
}

// Time: O(n) Space: O(n)
pub fn invert_binary_tree_depth_first_search_stack<T: Clone + Debug>(
    node: Option<Box<Tree<T>>>,
) -> Option<Box<Tree<T>>> {
    let mut tree = node?;
    let mut stack: Vec<&mut Tree<T>> = Vec::new();
    stack.push(&mut tree);

    while let Some(current) = stack.pop() {
        let Tree { left, right, .. } = current;
        std::mem::swap(left, right);

        if let Some(node) = left {
            stack.push(node);
        }
        if let Some(node) = right {
            stack.push(node);
        }
    }

    Some(tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    type Invert = fn(Option<Box<Tree<i64>>>) -> Option<Box<Tree<i64>>>;

    const INVERTS: [Invert; 3] = [
        invert_binary_tree_breadth_first_search,
        invert_binary_tree_depth_first_search,
        invert_binary_tree_depth_first_search_stack,
    ];

    fn check(root: &[i64], expected: &[i64]) {
        let tree = Tree::from_vec(root).map(Box::new);
        for invert in INVERTS {
            let inverted = invert(tree.clone()).map(|t| t.to_vec());
            assert_eq!(inverted.as_deref(), Some(expected));
        }
    }

    #[test]
    fn example_1() {
        check(&[1, 2, 3, 4, 5, 6, 7], &[1, 3, 2, 7, 6, 5, 4]);
    }

    #[test]
    fn example_2() {
        check(&[3, 2, 1], &[3, 1, 2]);
    }

    #[test]
    fn example_3() {
        assert!(Tree::<i64>::from_vec(&[]).is_none());
        for invert in INVERTS {
            assert!(invert(None).is_none());
        }
    }

    #[test]
    fn single_node() {
        check(&[9], &[9]);
    }

    #[test]
    fn incomplete_last_level() {
        // 1 has children 2 and 3; 2 has only a left child, 4.
        check(&[1, 2, 3, 4], &[1, 3, 2, 4]);
    }

    #[test]
    fn from_vec_round_trips() {
        let values = [1, 2, 3, 4, 5];
        assert_eq!(
            Tree::from_vec(&values).map(|t| t.to_vec()),
            Some(values.to_vec())
        );
    }
}
