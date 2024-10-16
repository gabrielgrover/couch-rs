use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    borrow::Cow,
    ops::{Index, IndexMut},
};

pub const ID_FIELD: &str = "_id";
pub const REV_FIELD: &str = "_rev";

/// Trait to deal with typed `CouchDB` documents.
/// For types implementing this trait, the _id and _rev fields on the json data sent/received to/from couchdb are automatically handled by this crate, using `get_id` and `get_rev` to get the values (before sending data to couchdb) and `set_id` and `set_rev` to set them (after receiving data from couchdb).
/// *Note*, when reading documents from couchdb directly, if whichever field name is used to store the revision is different from "_rev" (e.g. "`my_rev`"), the value will always be "the last value of _rev" as updating "_rev is handled by couchdb, not this crate. This should be transparent to users of this crate
/// because `set_rev` will be called before returning the document to the user, so the user will always see the correct value.
pub trait TypedCouchDocument: DeserializeOwned + Serialize + Sized {
    /// get the _id field
    fn get_id(&self) -> Cow<str>;
    /// get the _rev field
    fn get_rev(&self) -> Cow<str>;
    /// set the _rev field
    fn set_rev(&mut self, rev: &str);
    /// set the _id field
    fn set_id(&mut self, id: &str);
    /// merge the _id and _rev from the other document with this one
    fn merge_ids(&mut self, other: &Self);
}

/// Allows dealing with _id and _rev fields in untyped (Value) documents
impl TypedCouchDocument for Value {
    fn get_id(&self) -> Cow<str> {
        let id: String = json_extr!(self[ID_FIELD]);
        Cow::from(id)
    }

    fn get_rev(&self) -> Cow<str> {
        let rev: String = json_extr!(self[REV_FIELD]);
        Cow::from(rev)
    }

    fn set_rev(&mut self, rev: &str) {
        if let Some(o) = self.as_object_mut() {
            o.insert(REV_FIELD.to_string(), Value::from(rev));
        }
    }

    fn set_id(&mut self, id: &str) {
        if let Some(o) = self.as_object_mut() {
            o.insert(ID_FIELD.to_string(), Value::from(id));
        }
    }

    fn merge_ids(&mut self, other: &Self) {
        self.set_id(&other.get_id());
        self.set_rev(&other.get_rev());
    }
}

/// Memory-optimized, iterable document collection, mostly returned in calls
/// that involve multiple documents results Can target a specific index through
/// implementation of `Index` and `IndexMut`
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DocumentCollection<T: TypedCouchDocument> {
    pub offset: Option<u32>,
    pub rows: Vec<T>,
    pub total_rows: u32,
    pub bookmark: Option<String>,
}

impl<T: TypedCouchDocument> Default for DocumentCollection<T> {
    fn default() -> Self {
        DocumentCollection {
            offset: None,
            rows: vec![],
            total_rows: 0,
            bookmark: None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(bound(deserialize = "T: TypedCouchDocument"))]
pub struct AllDocsResponse<T: TypedCouchDocument> {
    pub total_rows: Option<u32>,
    pub offset: Option<u32>,
    pub rows: Vec<DocResponse<T>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(bound(deserialize = "T: TypedCouchDocument"))]
pub struct DocResponse<T: TypedCouchDocument> {
    pub id: Option<String>,
    pub key: Option<Value>,
    pub value: Option<DocResponseValue>,
    pub error: Option<String>,
    pub doc: Option<T>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct DocResponseValue {
    pub rev: String,
}

impl<T: TypedCouchDocument> DocumentCollection<T> {
    /// Create a new document collection from an `AllDocsResponse`
    ///
    /// # Panics
    /// Panics if the `total_rows` field is greater than `u32::MAX`
    #[must_use]
    pub fn new(doc: AllDocsResponse<T>) -> DocumentCollection<T> {
        let rows = doc.rows;
        let items: Vec<T> = rows
            .into_iter()
            .filter_map(|d| {
                if d.error.is_some() {
                    // remove errors
                    None
                } else {
                    // Remove _design documents
                    d.doc.filter(|doc| !doc.get_id().starts_with('_'))
                }
            })
            .collect();

        DocumentCollection {
            offset: doc.offset,
            total_rows: u32::try_from(items.len()).expect("total_rows > u32::MAX is not supported"),
            rows: items,
            bookmark: Option::None,
        }
    }

    /// Create a new document collection from a `Vec` of documents
    ///
    /// # Panics
    /// Panics if the `total_rows` field is greater than `u32::MAX`
    #[must_use]
    pub fn new_from_documents(docs: Vec<T>, bookmark: Option<String>) -> DocumentCollection<T> {
        let len = u32::try_from(docs.len()).expect("total_rows > u32::MAX is not supported");
        DocumentCollection {
            offset: Some(0),
            total_rows: len,
            rows: docs,
            bookmark,
        }
    }

    /// Create a new document collection from a `Vec` of `Value` documents
    ///
    /// # Panics
    /// Panics if the `total_rows` field is greater than `u32::MAX`
    #[must_use]
    pub fn new_from_values(docs: Vec<Value>, bookmark: Option<String>) -> DocumentCollection<T> {
        let len = u32::try_from(docs.len()).expect("total_rows > u32::MAX is not supported");

        DocumentCollection {
            offset: Some(0),
            total_rows: len,
            rows: docs
                .into_iter()
                .filter_map(|d| serde_json::from_value::<T>(d).ok())
                .collect(),
            bookmark,
        }
    }

    /// Returns raw JSON data from documents
    #[must_use]
    pub fn get_data(&self) -> &Vec<T> {
        &self.rows
    }
}

impl<T: TypedCouchDocument> Index<usize> for DocumentCollection<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.rows.get(index).unwrap()
    }
}

impl<T: TypedCouchDocument> IndexMut<usize> for DocumentCollection<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.rows.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate as couch_rs;
    use crate::document::TypedCouchDocument;
    use couch_rs_derive::CouchDocument;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, CouchDocument, Debug, Default)]
    struct TestDocument {
        #[serde(skip_serializing_if = "String::is_empty")]
        pub _id: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub _rev: String,
    }

    #[test]
    fn test_derive_couch_document() {
        let doc = TestDocument {
            _id: "1".to_string(),
            _rev: "2".to_string(),
        };
        let id = doc.get_id();
        let rev = doc.get_rev();
        assert_eq!(id, "1");
        assert_eq!(rev, "2");
    }
}
