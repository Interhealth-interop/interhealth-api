use bson::{doc, Document};

/// Build a MongoDB sort document from order field and order direction
/// 
/// # Arguments
/// * `order_field` - Optional field name to sort by
/// * `order_by` - Optional sort direction ("ASC" or "DESC")
/// 
/// # Returns
/// A MongoDB sort document, or None if no sorting is specified
pub fn build_sort_document(order_field: Option<String>, order_by: Option<String>) -> Option<Document> {
    if let Some(field) = order_field {
        let direction = match order_by.as_deref() {
            Some("DESC") | Some("desc") => -1,
            _ => 1, // Default to ASC
        };
        
        Some(doc! { field: direction })
    } else {
        None
    }
}
