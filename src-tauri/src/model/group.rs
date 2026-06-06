use serde::{Deserialize, Serialize};
use super::{CommonProps, Element};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub children: Vec<Element>,
    #[serde(default)]
    pub expanded: bool,
}
