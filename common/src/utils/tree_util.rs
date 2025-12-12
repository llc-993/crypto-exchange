use std::collections::HashMap;
use std::hash::Hash;

/// Tree node trait for building hierarchical structures
/// 
/// This trait defines the interface for any type that can be organized into a tree structure.
/// Implement this trait to enable automatic tree building using the utility functions.
pub trait TreeNode: Clone {
    /// The type used for node IDs (must be comparable and hashable)
    type Id: Eq + Hash + Copy;
    
    /// Get the node's unique identifier
    fn get_id(&self) -> Self::Id;
    
    /// Get the parent node's identifier (None for root nodes)
    fn get_parent_id(&self) -> Option<Self::Id>;
    
    /// Get mutable reference to the children vector
    fn get_children_mut(&mut self) -> &mut Vec<Self>;
    
    /// Get immutable reference to the children vector
    fn get_children(&self) -> &Vec<Self>;
    
    /// Get the sort key for ordering siblings (default implementation returns 0)
    fn get_sort_key(&self) -> i32 {
        0
    }
}

/// Build a tree structure from a flat list of nodes
/// 
/// # Arguments
/// * `nodes` - A vector of nodes implementing TreeNode trait
/// * `root_parent_id` - The parent ID value that identifies root nodes (typically 0)
/// 
/// # Returns
/// A vector of root nodes with their children recursively populated
/// 
/// # Example
/// ```rust
/// let nodes = vec![...]; // Your TreeNode implementations
/// let tree = build_tree(nodes, 0); // Build tree where root nodes have parent_id = 0
/// ```
pub fn build_tree<T: TreeNode>(
    nodes: Vec<T>,
    root_parent_id: T::Id,
) -> Vec<T> {
    if nodes.is_empty() {
        return Vec::new();
    }

    // 1. Separate root nodes from all nodes
    let mut root_nodes: Vec<T> = Vec::new();
    
    for node in nodes.iter() {
        if node.get_parent_id() == Some(root_parent_id) {
            root_nodes.push(node.clone());
        }
    }
    
    // 2. Group all nodes by their parent_id using HashMap
    let mut children_by_parent: HashMap<T::Id, Vec<T>> = HashMap::new();
    for node in nodes {
        if let Some(parent_id) = node.get_parent_id() {
            children_by_parent
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(node);
        }
    }
    
    // 3. Sort root nodes by their sort key
    root_nodes.sort_by(|a, b| a.get_sort_key().cmp(&b.get_sort_key()));
    
    // 4. Recursively build the tree
    attach_children(&mut root_nodes, &children_by_parent);
    
    root_nodes
}

/// Recursively attach children to parent nodes
/// 
/// This is an internal helper function that recursively populates the children
/// of each node using the pre-computed HashMap grouping.
fn attach_children<T: TreeNode>(
    nodes: &mut Vec<T>,
    children_by_parent: &HashMap<T::Id, Vec<T>>,
) {
    for node in nodes.iter_mut() {
        let node_id = node.get_id();
        
        // Find children for this node
        if let Some(children) = children_by_parent.get(&node_id) {
            if !children.is_empty() {
                // Clone children and sort them
                let mut sorted_children = children.clone();
                sorted_children.sort_by(|a, b| a.get_sort_key().cmp(&b.get_sort_key()));
                
                // Recursively attach grandchildren
                attach_children(&mut sorted_children, children_by_parent);
                
                // Set the sorted children
                *node.get_children_mut() = sorted_children;
            }
        }
    }
}

/// Build a tree structure with a custom comparator for sorting
/// 
/// # Arguments
/// * `nodes` - A vector of nodes implementing TreeNode trait
/// * `root_parent_id` - The parent ID value that identifies root nodes
/// * `comparator` - A function to compare two nodes for sorting
/// 
/// # Returns
/// A vector of root nodes with their children recursively populated and sorted
pub fn build_tree_with_comparator<T, F>(
    nodes: Vec<T>,
    root_parent_id: T::Id,
    comparator: F,
) -> Vec<T>
where
    T: TreeNode,
    F: Fn(&T, &T) -> std::cmp::Ordering + Copy,
{
    if nodes.is_empty() {
        return Vec::new();
    }

    // 1. Separate root nodes
    let mut root_nodes: Vec<T> = Vec::new();
    
    for node in nodes.iter() {
        if node.get_parent_id() == Some(root_parent_id) {
            root_nodes.push(node.clone());
        }
    }
    
    // 2. Group nodes by parent_id
    let mut children_by_parent: HashMap<T::Id, Vec<T>> = HashMap::new();
    for node in nodes {
        if let Some(parent_id) = node.get_parent_id() {
            children_by_parent
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(node);
        }
    }
    
    // 3. Sort root nodes with custom comparator
    root_nodes.sort_by(comparator);
    
    // 4. Recursively build with custom comparator
    attach_children_with_comparator(&mut root_nodes, &children_by_parent, comparator);
    
    root_nodes
}

/// Recursively attach children with custom sorting
fn attach_children_with_comparator<T, F>(
    nodes: &mut Vec<T>,
    children_by_parent: &HashMap<T::Id, Vec<T>>,
    comparator: F,
)
where
    T: TreeNode,
    F: Fn(&T, &T) -> std::cmp::Ordering + Copy,
{
    for node in nodes.iter_mut() {
        let node_id = node.get_id();
        
        if let Some(children) = children_by_parent.get(&node_id) {
            if !children.is_empty() {
                let mut sorted_children = children.clone();
                sorted_children.sort_by(comparator);
                
                attach_children_with_comparator(&mut sorted_children, children_by_parent, comparator);
                
                *node.get_children_mut() = sorted_children;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestNode {
        id: i64,
        parent_id: Option<i64>,
        name: String,
        sort_key: i32,
        children: Vec<TestNode>,
    }

    impl TreeNode for TestNode {
        type Id = i64;

        fn get_id(&self) -> Self::Id {
            self.id
        }

        fn get_parent_id(&self) -> Option<Self::Id> {
            self.parent_id
        }

        fn get_children_mut(&mut self) -> &mut Vec<Self> {
            &mut self.children
        }

        fn get_children(&self) -> &Vec<Self> {
            &self.children
        }

        fn get_sort_key(&self) -> i32 {
            self.sort_key
        }
    }

    #[test]
    fn test_build_tree_simple() {
        let nodes = vec![
            TestNode { id: 1, parent_id: Some(0), name: "Root1".to_string(), sort_key: 1, children: vec![] },
            TestNode { id: 2, parent_id: Some(1), name: "Child1".to_string(), sort_key: 1, children: vec![] },
            TestNode { id: 3, parent_id: Some(1), name: "Child2".to_string(), sort_key: 2, children: vec![] },
        ];

        let tree = build_tree(nodes, 0);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].id, 1);
        assert_eq!(tree[0].children.len(), 2);
        assert_eq!(tree[0].children[0].id, 2);
        assert_eq!(tree[0].children[1].id, 3);
    }

    #[test]
    fn test_build_tree_sorting() {
        let nodes = vec![
            TestNode { id: 1, parent_id: Some(0), name: "Root1".to_string(), sort_key: 2, children: vec![] },
            TestNode { id: 2, parent_id: Some(0), name: "Root2".to_string(), sort_key: 1, children: vec![] },
        ];

        let tree = build_tree(nodes, 0);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].id, 2); // sort_key=1 comes first
        assert_eq!(tree[1].id, 1); // sort_key=2 comes second
    }
}
