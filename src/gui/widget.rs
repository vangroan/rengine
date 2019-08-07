use daggy::NodeIndex;

pub type WidgetIndexType = u32;

/// Identifier for a Widget.
///
/// Wraps the graph node index, and acts as
/// a reference back to a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetId(NodeIndex<WidgetIndexType>);

impl WidgetId {
    pub(crate) fn node_index(&self) -> NodeIndex<WidgetIndexType> {
        self.0
    }
}

impl From<WidgetIndexType> for WidgetId {
    fn from(index: WidgetIndexType) -> Self {
        WidgetId(index.into())
    }
}

impl From<NodeIndex<WidgetIndexType>> for WidgetId {
    fn from(node_index: NodeIndex<WidgetIndexType>) -> Self {
        WidgetId(node_index)
    }
}
