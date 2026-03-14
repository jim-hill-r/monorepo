// You are given the root of a binary tree root. Invert the binary tree and return its root.

// Recommended complexity: Time: O(n) Space: O(n)

// Hint 1: From the diagram, you can see that the left and right children of every node in the tree are swapped. Can you think of a way to achieve this recursively? Maybe an algorithm that is helpful to traverse the tree.

// Hint 2: We can use the Depth First Search (DFS) algorithm. At each node, we swap its left and right children by swapping their pointers. This inverts the current node, but every node in the tree also needs to be inverted. To achieve this, we recursively visit the left and right children and perform the same operation. If the current node is null, we simply return.

use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct Tree<T: Clone + Debug> {
    value: T,
    left: Option<Box<Tree<T>>>,
    right: Option<Box<Tree<T>>>,
}

impl<T: Clone + Debug> Tree<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }

    fn from_vec(values: &Vec<T>) -> Option<Tree<T>> {
        if values.len() < 1 {
            return None;
        }

        let mut root: Tree<T> = Self::new(values[0].clone());
        for i in 1..values.len() {
            root.insert(values[i].clone());
        }
        return Some(root);
    }

    // Breadth first traverse
    fn to_vec(&mut self) -> Vec<T> {
        let mut ret: Vec<T> = vec![];
        let mut queue: VecDeque<&mut Tree<T>> = VecDeque::new();
        queue.push_front(self);

        while queue.len() > 0 {
            let current = queue.pop_back().unwrap();
            ret.push(current.value.clone());
            let Tree { left, right, .. } = current;
            if let Some(node) = left {
                queue.push_front(node);
            }
            if let Some(node) = right {
                queue.push_front(node);
            }
        }

        return ret;
    }

    fn insert(&mut self, value: T) {
        let mut queue: VecDeque<&mut Tree<T>> = VecDeque::new();
        queue.push_front(self);

        loop {
            let current = queue.pop_back().unwrap();
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
fn invert_binary_tree_breadth_first_search<T: Clone + Debug>(
    node: Option<Box<Tree<T>>>,
) -> Option<Box<Tree<T>>> {
    let mut tree = node?;
    let mut queue: VecDeque<&mut Tree<T>> = VecDeque::new();
    queue.push_front(&mut tree);

    while queue.len() > 0 {
        let current = queue.pop_back().unwrap();
        let Tree { left, right, .. } = current;
        std::mem::swap(left, right);

        if let Some(node) = left {
            queue.push_front(node);
        }
        if let Some(node) = right {
            queue.push_front(node);
        }
    }

    return Some(tree);
}

// Time: O(n) Space: O(n)
fn invert_binary_tree_depth_first_search<T: Clone + Debug>(
    node: Option<Box<Tree<T>>>,
) -> Option<Box<Tree<T>>> {
    let tree = node?;
    let mut new_tree = Tree::new(tree.value);
    new_tree.left = invert_binary_tree_depth_first_search(tree.right);
    new_tree.right = invert_binary_tree_depth_first_search(tree.left);

    return Some(Box::new(new_tree));
}

// Time: O() Space: O()
fn invert_binary_tree_depth_first_search_stack<T: Clone + Debug>(
    node: Option<Box<Tree<T>>>,
) -> Option<Box<Tree<T>>> {
    let mut tree = node?;
    let mut stack: Vec<&mut Tree<T>> = Vec::new();
    stack.push(&mut tree);

    while stack.len() > 0 {
        let current = stack.pop().unwrap();
        let Tree { left, right, .. } = current;
        std::mem::swap(left, right);

        if let Some(node) = left {
            stack.push(node);
        }
        if let Some(node) = right {
            stack.push(node);
        }
    }

    return Some(tree);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let root = vec![1, 2, 3, 4, 5, 6, 7];
        let expected = vec![1, 3, 2, 7, 6, 5, 4];
        let tree = Tree::from_vec(&root).unwrap();
        assert_eq!(
            invert_binary_tree_breadth_first_search(Some(Box::new(tree.clone())))
                .unwrap()
                .to_vec(),
            expected
        );
        assert_eq!(
            invert_binary_tree_depth_first_search(Some(Box::new(tree.clone())))
                .unwrap()
                .to_vec(),
            expected
        );
        assert_eq!(
            invert_binary_tree_depth_first_search_stack(Some(Box::new(tree.clone())))
                .unwrap()
                .to_vec(),
            expected
        );
    }

    #[test]
    fn example_2() {
        let root = vec![3, 2, 1];
        let expected = vec![3, 1, 2];
        let tree = Tree::from_vec(&root).unwrap();
        assert_eq!(
            invert_binary_tree_breadth_first_search(Some(Box::new(tree.clone())))
                .unwrap()
                .to_vec(),
            expected
        );
        assert_eq!(
            invert_binary_tree_depth_first_search(Some(Box::new(tree.clone())))
                .unwrap()
                .to_vec(),
            expected
        );
        assert_eq!(
            invert_binary_tree_depth_first_search_stack(Some(Box::new(tree.clone())))
                .unwrap()
                .to_vec(),
            expected
        );
    }

    #[test]
    fn example_3() {
        let root: Vec<i64> = vec![];
        //let expected = vec![];
        let tree = Tree::from_vec(&root);
        assert!(tree.is_none());
    }
}
