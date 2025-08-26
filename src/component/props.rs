use std::any::Any;

/// Trait for component properties.
/// Props must be cloneable and comparable for efficient diffing.
pub trait Props: Clone + PartialEq + Send + Sync + 'static {
    /// Create default props
    fn default_props() -> Self
    where
        Self: Sized + Default,
    {
        Self::default()
    }

    /// Convert to Any for type erasure
    fn as_any(&self) -> &dyn Any;
}

/// Empty props for components that don't need properties
#[derive(Clone, PartialEq, Debug, Default)]
pub struct EmptyProps;

impl Props for EmptyProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Common props that many components might use
#[derive(Clone, PartialEq, Debug)]
pub struct CommonProps {
    pub id: Option<String>,
    pub class: Option<String>,
    pub visible: bool,
    pub focusable: bool,
}

impl Default for CommonProps {
    fn default() -> Self {
        Self {
            id: None,
            class: None,
            visible: true,
            focusable: true,
        }
    }
}

impl Props for CommonProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Props with children support
#[derive(Clone, PartialEq)]
pub struct PropsWithChildren<P: Props> {
    pub props: P,
    pub children: Vec<crate::component::Element>,
}

impl<P: Props> Props for PropsWithChildren<P> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
