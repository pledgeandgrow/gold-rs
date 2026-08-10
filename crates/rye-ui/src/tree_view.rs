//! TreeView — collapsible nested tree.

use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub icon: Option<String>,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            expanded: false,
            children: Vec::new(),
        }
    }
    pub fn icon(mut self, i: impl Into<String>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn expanded(mut self) -> Self {
        self.expanded = true;
        self
    }
    pub fn children(mut self, c: Vec<TreeNode>) -> Self {
        self.children = c;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TreeViewProps {
    pub root: TreeNode,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TreeViewProps {
    fn default() -> Self {
        Self {
            root: TreeNode::new("Root"),
            class: None,
            style: None,
        }
    }
}

impl TreeViewProps {
    pub fn root(mut self, r: TreeNode) -> Self {
        self.root = r;
        self
    }
}

pub struct TreeView;

impl TreeView {
    pub fn render(props: TreeViewProps) -> Element {
        let node = render_node(&props.root, 0);
        let style = format!(
            "font-size:14px;color:#1e293b;user-select:none;{}",
            props.style.as_deref().unwrap_or("")
        );

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), style),
                (
                    "class".to_string(),
                    format!("rye-tree-view {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            vec![node],
        ))
    }
}

fn render_node(node: &TreeNode, depth: usize) -> Template {
    let indent = depth * 20;
    let has_children = !node.children.is_empty();
    let arrow = if has_children {
        if node.expanded {
            "▼"
        } else {
            "▶"
        }
    } else {
        " "
    };

    let row_style = format!(
        "display:flex;align-items:center;gap:4px;padding:4px 8px;cursor:{};\
         border-radius:4px;padding-left:{}px;",
        if has_children { "pointer" } else { "default" },
        indent + 8,
    );

    let mut row_children = vec![Template::new_element(
        "span",
        vec![(
            "style".to_string(),
            "font-size:10px;color:#64748b;width:12px;".to_string(),
        )],
        Vec::new(),
        vec![Template::text(arrow)],
    )];

    if let Some(icon) = &node.icon {
        row_children.push(Template::new_element(
            "span",
            vec![("style".to_string(), "font-size:16px;".to_string())],
            Vec::new(),
            vec![Template::text(icon)],
        ));
    }

    row_children.push(Template::text(&node.label));

    let mut children = vec![Template::new_element(
        "div",
        vec![
            ("style".to_string(), row_style),
            ("class".to_string(), "rye-tree-node".to_string()),
        ],
        Vec::new(),
        row_children,
    )];

    if node.expanded && has_children {
        for child in &node.children {
            children.push(render_node(child, depth + 1));
        }
    }

    Template::new_element(
        "div",
        vec![("class".to_string(), "rye-tree-node-wrapper".to_string())],
        Vec::new(),
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_new() {
        let n = TreeNode::new("Root");
        assert_eq!(n.label, "Root");
        assert!(!n.expanded);
        assert!(n.children.is_empty());
    }

    #[test]
    fn test_tree_node_builder() {
        let n = TreeNode::new("src").icon("📁").expanded().children(vec![
            TreeNode::new("main.rs").icon("📄"),
            TreeNode::new("lib.rs").icon("📄"),
        ]);
        assert!(n.expanded);
        assert_eq!(n.children.len(), 2);
    }

    #[test]
    fn test_tree_view_render() {
        let el = TreeView::render(TreeViewProps::default().root(
            TreeNode::new("Project").expanded().children(vec![
                    TreeNode::new("src").expanded()
                        .children(vec![TreeNode::new("main.rs")]),
                    TreeNode::new("tests"),
                ]),
        ));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_tree_view_render_collapsed() {
        let el = TreeView::render(
            TreeViewProps::default()
                .root(TreeNode::new("Root").children(vec![TreeNode::new("Child")])),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
