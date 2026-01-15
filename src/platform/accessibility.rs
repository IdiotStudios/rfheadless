/// Accessibility tree representation for testing and parity checks

#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityNode {
    pub id: String,
    pub role: String,
    pub name: Option<String>,
    pub bounds: Option<(i32, i32, u32, u32)>,
    pub children: Vec<AccessibilityNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityTree {
    pub root_id: Option<String>,
    pub nodes: Vec<AccessibilityNode>,
}

pub trait AccessibilityProvider: Send + Sync {
    /// Export a reproducible accessibility tree snapshot for tests
    fn export_tree(&self) -> AccessibilityTree;
}

/// Noop provider that returns an empty tree
pub struct NoopAccessibility;

impl NoopAccessibility {
    pub fn new() -> Self {
        NoopAccessibility
    }
}

impl Default for NoopAccessibility {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityProvider for NoopAccessibility {
    fn export_tree(&self) -> AccessibilityTree {
        AccessibilityTree {
            root_id: None,
            nodes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_accessibility_exports_empty_tree() {
        let a = NoopAccessibility::new();
        let t = a.export_tree();
        assert!(t.nodes.is_empty());
        assert!(t.root_id.is_none());
    }
}
