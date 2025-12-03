
/// Represents a delete operation for removing a collection.
///
/// # Fields
///
/// * `collection_name` - The name of the collection to delete from
///
/// # Examples
///
/// ```
/// use vector_xlite::types::DeleteCollection;
///
/// let delete_collection = DeleteCollection::builder()
///     .collection_name("products")
///     .build()
///     .expect("Failed to build delete point");
/// ```
#[derive(Debug, Clone)]
pub struct DeleteCollection {
    pub collection_name: String,
}

impl DeleteCollection {
    /// Creates a new builder for constructing a DeleteCollection.
    pub fn builder() -> DeleteCollectionBuilder {
        DeleteCollectionBuilder::default()
    }
}

/// Builder for constructing DeleteCollection instances with validation.
#[derive(Debug, Default)]
pub struct DeleteCollectionBuilder {
    collection_name: Option<String>,
}

impl DeleteCollectionBuilder {
    /// Sets the collection name for the delete operation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection
    pub fn collection_name<S: Into<String>>(mut self, name: S) -> Self {
        self.collection_name = Some(name.into());
        self
    }

    /// Builds the DeleteCollection with validation.
    ///
    /// # Returns
    ///
    /// * `Ok(DeleteCollection)` - Successfully built DeleteCollection
    /// * `Err(String)` - Validation error message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * `collection_name` is not provided
    pub fn build(self) -> Result<DeleteCollection, String> {
        // Validate collection_name
        let collection_name = self
            .collection_name
            .ok_or_else(|| "Collection name must be provided.".to_string())?;

        Ok(DeleteCollection {
            collection_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_collection_builder_success() {
        let delete_collection = DeleteCollection::builder()
            .collection_name("test_collection")
            .build();

        assert!(delete_collection.is_ok());
        let point = delete_collection.unwrap();
        assert_eq!(point.collection_name, "test_collection");
    }

    #[test]
    fn test_delete_collection_builder_missing_collection_name() {
        let delete_collection = DeleteCollection::builder()
            .build();

        assert!(delete_collection.is_err());
        assert_eq!(delete_collection.unwrap_err(), "Collection name must be provided.");
    }
}
