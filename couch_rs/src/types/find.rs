use crate::document::TypedCouchDocument;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

/// Sort direction abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum SortDirection {
    #[serde(rename = "desc")]
    Desc,
    #[serde(rename = "asc")]
    Asc,
}

impl From<String> for SortDirection {
    fn from(original: String) -> SortDirection {
        match original.as_ref() {
            "desc" => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    }
}

/// Sort spec content abstraction
pub type SortSpecContent = HashMap<String, SortDirection>;

/// Sort spec abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum SortSpec {
    Simple(String),
    Complex(SortSpecContent),
}

/// Index spec abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum IndexSpec {
    DesignDocument(String),
    IndexName((String, String)),
}

/// Find query abstraction
/// Parameters here [/db/_find](https://docs.couchdb.org/en/latest/api/database/find.html)
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct FindQuery {
    pub selector: Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<u64>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sort: Vec<SortSpec>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_index: Option<IndexSpec>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_stats: Option<bool>,
}

/// Find result abstraction
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(bound(deserialize = "T: TypedCouchDocument"))]
pub struct FindResult<T: TypedCouchDocument> {
    pub docs: Option<Vec<T>>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>,
    pub bookmark: Option<String>,
}

//todo: include status on structs

/// Explain result abstraction
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ExplainResult {
    pub dbname: String,
    pub index: IndexSpec,
    pub selector: Value,
    pub opts: Value,
    pub limit: u32,
    pub skip: u64,
    pub fields: Vec<String>,
    pub range: Value,
}

/// $ne operation
#[derive(Serialize, Deserialize)]
pub struct NotEqual {
    #[serde(rename = "$ne")]
    pub ne: Option<String>,
}

/// Select all Selector
#[derive(Serialize, Deserialize)]
pub struct SelectAll {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<NotEqual>,
}

/// Little helper to create a select all query.
impl Default for SelectAll {
    fn default() -> Self {
        SelectAll {
            id: Some(NotEqual { ne: None }),
        }
    }
}

impl SelectAll {
    #[must_use]
    pub fn as_value(&self) -> Value {
        self.into()
    }
}

impl From<&SelectAll> for serde_json::Value {
    fn from(s: &SelectAll) -> Self {
        serde_json::to_value(s).expect("can not convert into json")
    }
}

impl From<serde_json::Value> for SelectAll {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).expect("json Value is not a valid Selector")
    }
}

/// Returns all documents
#[macro_export]
macro_rules! find_all_selector {
    () => {
        FindQuery::find_all().as_value()
    };
}

/// Find query. You can use the builder paradigm to construct these parameters easily:
/// ```
/// use couch_rs::types::find::FindQuery;
/// let _query = FindQuery::find_all().skip(10).limit(10);
/// ```
impl FindQuery {
    #[must_use]
    pub fn new_from_value(query: Value) -> Self {
        query.into()
    }

    // Create a new FindQuery from a valid selector. The selector syntax is documented here:
    // https://docs.couchdb.org/en/latest/api/database/find.html#find-selectors
    #[must_use]
    pub fn new(selector: Value) -> Self {
        FindQuery {
            selector,
            limit: None,
            skip: None,
            sort: vec![],
            fields: None,
            use_index: None,
            r: None,
            bookmark: None,
            update: None,
            stable: None,
            stale: None,
            execution_stats: None,
        }
    }

    #[must_use]
    pub fn find_all() -> Self {
        Self::new(SelectAll::default().as_value())
    }

    #[must_use]
    pub fn as_value(&self) -> Value {
        self.into()
    }

    #[must_use]
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn skip(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }

    #[must_use]
    pub fn sort(mut self, sort: Vec<SortSpec>) -> Self {
        self.sort = sort;
        self
    }

    #[must_use]
    pub fn fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    #[must_use]
    pub fn use_index(mut self, use_index: IndexSpec) -> Self {
        self.use_index = Some(use_index);
        self
    }

    #[must_use]
    pub fn r(mut self, r: i32) -> Self {
        self.r = Some(r);
        self
    }

    #[must_use]
    pub fn bookmark(mut self, bookmark: &str) -> Self {
        self.bookmark = Some(bookmark.to_string());
        self
    }

    #[must_use]
    pub fn update(mut self, update: bool) -> Self {
        self.update = Some(update);
        self
    }

    #[must_use]
    pub fn stable(mut self, stable: bool) -> Self {
        self.stable = Some(stable);
        self
    }

    #[must_use]
    pub fn stale(mut self, stale: &str) -> Self {
        self.stale = Some(stale.to_string());
        self
    }

    #[must_use]
    pub fn execution_stats(mut self, execution_stats: bool) -> Self {
        self.execution_stats = Some(execution_stats);
        self
    }
}

impl From<FindQuery> for serde_json::Value {
    fn from(q: FindQuery) -> Self {
        serde_json::to_value(q).expect("can not convert into json")
    }
}

impl From<&FindQuery> for serde_json::Value {
    fn from(q: &FindQuery) -> Self {
        serde_json::to_value(q).expect("can not convert into json")
    }
}

impl From<serde_json::Value> for FindQuery {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).expect("json Value is not a valid FindQuery")
    }
}

impl Display for FindQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json: Value = self.into();
        f.write_str(&json.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_convert_to_value() {
        let mut sort = HashMap::new();
        sort.insert("first_name".to_string(), SortDirection::Desc);

        let mut query = FindQuery::find_all();
        query.limit = Some(10);
        query.skip = Some(20);
        query.sort = vec![SortSpec::Complex(sort)];
        let json = query.to_string();
        assert_eq!(
            r#"{"limit":10,"selector":{"_id":{"$ne":null}},"skip":20,"sort":[{"first_name":"desc"}]}"#,
            json
        );
    }

    #[test]
    fn test_default_select_all() {
        let selector = FindQuery::find_all().as_value().to_string();
        assert_eq!(selector, r#"{"selector":{"_id":{"$ne":null}}}"#);
    }

    #[test]
    fn test_from_json() {
        let query = FindQuery::new_from_value(json!({
            "selector": {
                "thing": true
            },
            "limit": 1,
            "sort": [{
                "thing": "desc"
            }]
        }));

        let selector = query.selector.to_string();
        assert_eq!(selector, r#"{"thing":true}"#);
        assert_eq!(query.limit, Some(1));
        assert_eq!(query.sort.len(), 1);
        let first_sort = query.sort.first().unwrap();
        if let SortSpec::Complex(spec) = first_sort {
            assert!(spec.contains_key("thing"));
            let direction = spec.get("thing").unwrap();
            assert_eq!(direction, &SortDirection::Desc);
        } else {
            panic!("unexpected sort spec");
        }
    }
}
