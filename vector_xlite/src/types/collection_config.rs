use crate::{helper::sql_helpers::parse_collection_name, types::enums::DistanceFunction};

pub struct CollectionConfig {
    pub collection_name: String,
    pub dimension: u16,
    pub distance: DistanceFunction,
    pub index_file_path: Option<String>,
    pub max_elements: Option<u32>,
    pub payload_table_schema: Option<String>,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            collection_name: "".to_string(),
            dimension: 3,
            distance: DistanceFunction::Cosine,
            payload_table_schema: None,
            index_file_path: None,
            max_elements: Some(100000),
        }
    }
}

impl CollectionConfig {
    pub fn builder() -> CollectionConfigBuilder {
        CollectionConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct CollectionConfigBuilder {
    dimension: Option<u16>,
    distance: Option<DistanceFunction>,
    index_file_path: Option<String>,
    max_elements: Option<u32>,
    name: Option<String>,
    payload_table_schema: Option<String>,
}

impl CollectionConfigBuilder {
    pub fn collection_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn vector_dimension(mut self, dim: u16) -> Self {
        self.dimension = Some(dim);
        self
    }

    pub fn distance(mut self, dist: DistanceFunction) -> Self {
        self.distance = Some(dist);
        self
    }

    pub fn payload_table_schema<S: Into<String>>(mut self, schema: S) -> Self {
        self.payload_table_schema = Some(schema.into());
        self
    }

    pub fn index_file_path<S: Into<String>>(mut self, path: S) -> Self {
        self.index_file_path = Some(path.into());
        self
    }

    pub fn max_elements(mut self, max_elems: u32) -> Self {
        self.max_elements = Some(max_elems);
        self
    }

    pub fn build(self) -> Result<CollectionConfig, &'static str> {
        if self.name.is_none() && self.payload_table_schema.is_none() {
            return Err("Either collection_name or payload_table_schema must be provided.".into());
        }

        let default = CollectionConfig::default();

        Ok(CollectionConfig {
            collection_name: parse_collection_name(self.payload_table_schema.as_ref())
                .unwrap_or(self.name.unwrap_or(default.collection_name)),
            dimension: self.dimension.unwrap_or(default.dimension),
            distance: self.distance.unwrap_or(default.distance),
            payload_table_schema: self.payload_table_schema.or(default.payload_table_schema),
            index_file_path: self.index_file_path.or(default.index_file_path),
            max_elements: self.max_elements.or(default.max_elements),
        })
    }
}
