use crate::helper::sql_helpers::parse_collection_name;

#[derive(Debug, Clone)]
pub struct SearchPoint {
    pub collection_name: String,
    pub vector: Vec<f32>,
    pub top_k: i64,
    pub payload_search_query: Option<String>,
}

impl SearchPoint {
    pub fn builder() -> SearchPointBuilder {
        SearchPointBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct SearchPointBuilder {
    collection_name: Option<String>,
    vector: Option<Vec<f32>>,
    top_k: Option<i64>,
    payload_search_query: Option<String>,
}

impl SearchPointBuilder {
    pub fn collection_name<S: Into<String>>(mut self, name: S) -> Self {
        self.collection_name = Some(name.into());
        self
    }

    pub fn vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }

    pub fn top_k(mut self, top_k: i64) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn payload_search_query<S: Into<String>>(mut self, query: S) -> Self {
        self.payload_search_query = Some(query.into());
        self
    }

    /// ✅ Build with validation:
    /// - Requires vector
    /// - top_k must be positive
    /// - Either collection_name or payload_search_query must be provided
    pub fn build(self) -> Result<SearchPoint, String> {
        if self.collection_name.is_none() && self.payload_search_query.is_none() {
            return Err("Either collection_name or payload_search_query must be provided.".into());
        }

        let vector = self
            .vector
            .ok_or_else(|| "Vector must be provided.".to_string())?;

        let top_k = self.top_k.unwrap_or(10);
        if top_k <= 0 {
            return Err("top_k must be greater than 0.".into());
        }

        Ok(SearchPoint {
            collection_name: parse_collection_name(self.payload_search_query.as_ref())
                .unwrap_or(self.collection_name.unwrap_or("".to_string())),
            vector,
            top_k,
            payload_search_query: self.payload_search_query,
        })
    }
}
