/// Represents a delete operation for removing a vector from a collection.
///
/// # Fields
///
/// * `collection_name` - The name of the collection to delete from
/// * `id` - The unique identifier of the vector to delete
///
/// # Examples
///
/// ```
/// use vector_xlite::types::DeletePoint;
///
/// let delete_point = DeletePoint::builder()
///     .collection_name("products")
///     .id(42)
///     .build()
///     .expect("Failed to build delete point");
/// ```
#[derive(Debug, Clone)]
pub struct DeletePoint {
    pub collection_name: String,
    pub id: u64,
}

impl DeletePoint {
    /// Creates a new builder for constructing a DeletePoint.
    pub fn builder() -> DeletePointBuilder {
        DeletePointBuilder::default()
    }
}

/// Builder for constructing DeletePoint instances with validation.
#[derive(Debug, Default)]
pub struct DeletePointBuilder {
    collection_name: Option<String>,
    id: Option<u64>,
}

impl DeletePointBuilder {
    /// Sets the collection name for the delete operation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection
    pub fn collection_name<S: Into<String>>(mut self, name: S) -> Self {
        self.collection_name = Some(name.into());
        self
    }

    /// Sets the ID of the vector to delete.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the vector
    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// Builds the DeletePoint with validation.
    ///
    /// # Returns
    ///
    /// * `Ok(DeletePoint)` - Successfully built DeletePoint
    /// * `Err(String)` - Validation error message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * `collection_name` is not provided
    /// * `id` is not provided
    pub fn build(self) -> Result<DeletePoint, String> {
        // Validate collection_name
        let collection_name = self
            .collection_name
            .ok_or_else(|| "Collection name must be provided.".to_string())?;

        // Validate ID
        let id = self
            .id
            .ok_or_else(|| "ID must be provided.".to_string())?;

        Ok(DeletePoint {
            collection_name,
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_point_builder_success() {
        let delete_point = DeletePoint::builder()
            .collection_name("test_collection")
            .id(123)
            .build();

        assert!(delete_point.is_ok());
        let point = delete_point.unwrap();
        assert_eq!(point.collection_name, "test_collection");
        assert_eq!(point.id, 123);
    }

    #[test]
    fn test_delete_point_builder_missing_collection_name() {
        let delete_point = DeletePoint::builder()
            .id(123)
            .build();

        assert!(delete_point.is_err());
        assert_eq!(delete_point.unwrap_err(), "Collection name must be provided.");
    }

    #[test]
    fn test_delete_point_builder_missing_id() {
        let delete_point = DeletePoint::builder()
            .collection_name("test")
            .build();

        assert!(delete_point.is_err());
        assert_eq!(delete_point.unwrap_err(), "ID must be provided.");
    }
}
