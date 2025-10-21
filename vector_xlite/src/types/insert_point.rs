
use crate::helper::sql_helpers::parse_collection_name;

#[derive(Debug, Clone)]
pub struct InsertPoint {
    pub collection_name: String,
    pub id: Option<u64>,
    pub vector: Vec<f32>,
    pub payload_insert_query: Option<String>,
}

impl InsertPoint {
    pub fn builder() -> InsertPointBuilder {
        InsertPointBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct InsertPointBuilder {
    collection_name: Option<String>,
    id: Option<u64>,
    vector: Option<Vec<f32>>,
    payload_insert_query: Option<String>,
}

impl InsertPointBuilder {
    pub fn collection_name<S: Into<String>>(mut self, name: S) -> Self {
        self.collection_name = Some(name.into());
        self
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }

    pub fn payload_insert_query<S: Into<String>>(mut self, query: S) -> Self {
        self.payload_insert_query = Some(query.into());
        self
    }

    /// ✅ Build with validation:
    /// Ensures that either `collection_name` or `payload_insert_query` is provided.
    pub fn build(self) -> Result<InsertPoint, String> {
        // Validate collection_name / payload_insert_query rule
        if self.collection_name.is_none() && self.payload_insert_query.is_none() {
            return Err("Either collection_name or payload_insert_query must be provided.".into());
        }

        // Validate vector presence
        let vector = self
            .vector
            .ok_or_else(|| "Vector must be provided.".to_string())?;

        Ok(InsertPoint {
            collection_name: parse_collection_name(self.payload_insert_query.as_ref())
                .unwrap_or(self.collection_name.unwrap_or("".to_string())),
            id: self.id,
            vector,
            payload_insert_query: self.payload_insert_query,
        })
    }
}
