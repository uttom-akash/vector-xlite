use crate::proto::{CollectionConfigPb, InsertPointPb, SearchPointPb};
use std::convert::TryFrom;
use vector_xlite::types::{
    CollectionConfig, CollectionConfigBuilder, DistanceFunction, InsertPoint, SearchPoint,
};

impl TryFrom<CollectionConfigPb> for CollectionConfig {
    type Error = String;
    fn try_from(pb: CollectionConfigPb) -> Result<Self, Self::Error> {
        let distance = match pb.distance.to_lowercase().as_str() {
            "cosine" => DistanceFunction::Cosine,
            "l2" => DistanceFunction::L2,
            "ip" => DistanceFunction::IP,
            other => return Err(format!("unknown distance: {}", other)),
        };

        CollectionConfigBuilder::default()
            .collection_name(&pb.collection_name)
            .distance(distance)
            .vector_dimension(pb.vector_dimension as u16)
            .payload_table_schema(&pb.payload_table_schema)
            .index_file_path(pb.index_file_path)
            .build()
            .map_err(|e| e.to_string())
    }
}

impl TryFrom<InsertPointPb> for InsertPoint {
    type Error = String;
    fn try_from(pb: InsertPointPb) -> Result<Self, Self::Error> {
        let mut b = InsertPoint::builder()
            .collection_name(&pb.collection_name)
            .id(pb.id as u64)
            .vector(pb.vector);

        if !pb.payload_insert_query.is_empty() {
            b = b.payload_insert_query(&pb.payload_insert_query);
        }
        b.build().map_err(|e| e.to_string())
    }
}

impl TryFrom<SearchPointPb> for SearchPoint {
    type Error = String;
    fn try_from(pb: SearchPointPb) -> Result<Self, Self::Error> {
        let mut b: vector_xlite::types::SearchPointBuilder = SearchPoint::builder();
        b = b.collection_name(&pb.collection_name);
        b = b.vector(pb.vector);
        b = b.top_k(pb.top_k as i64);
        if !pb.payload_search_query.is_empty() {
            b = b.payload_search_query(&pb.payload_search_query);
        }
        b.build().map_err(|e| e.to_string())
    }
}

// Conversion helpers for responses
use crate::proto::{KeyValuePb, SearchResultItemPb};
use std::collections::HashMap;

pub fn map_payload_to_kvs(map: &HashMap<String, String>) -> Vec<KeyValuePb> {
    map.iter()
        .map(|(k, v)| KeyValuePb {
            key: k.clone(),
            value: v.clone(),
        })
        .collect()
}

pub fn build_search_item(
    rowid: i64,
    distance: f32,
    payload: HashMap<String, String>,
) -> SearchResultItemPb {
    SearchResultItemPb {
        rowid,
        distance,
        payload: map_payload_to_kvs(&payload),
    }
}
