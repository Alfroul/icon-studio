use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An entry (single icon) within an icon set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct SetEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub project_path: String,
    #[serde(default)]
    pub thumbnail: String,
}

/// A collection of icons with unified style.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct IconSet {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub entries: Vec<SetEntry>,
    #[serde(default)]
    pub created_at: String,
}

/// A consistency issue found when checking an icon set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ConsistencyIssue {
    pub property: String,
    pub expected: String,
    pub actual: String,
    pub element_id: String,
    pub project_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_set_serde_roundtrip() {
        let set = IconSet {
            id: "set-1".to_string(),
            name: "Navigation Icons".to_string(),
            description: "A set of nav icons".to_string(),
            entries: vec![SetEntry {
                id: "entry-1".to_string(),
                name: "home".to_string(),
                tags: vec!["navigation".to_string()],
                project_path: "/icons/home.iconproject.json".to_string(),
                thumbnail: String::new(),
            }],
            created_at: "2025-01-01".to_string(),
        };
        let json = serde_json::to_string(&set).unwrap();
        let parsed: IconSet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, set);
    }
}
