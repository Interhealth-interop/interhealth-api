use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};

/// Format a datetime string to ISO 8601 format (YYYY-MM-DDTHH:MM:SS)
/// Handles various input formats and ensures consistent output for FHIR resources
pub fn format_to_iso8601(value: &str) -> String {
    // If already in ISO 8601 format with 'T', return as is
    if value.contains('T') {
        return value.to_string();
    }
    
    // Try to parse various datetime formats
    // Format 1: RFC3339 (already handled above)
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return dt.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    
    // Format 2: "%d-%m-%Y %H:%M:%S" (e.g., "11-08-2025 16:08:39")
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%d-%m-%Y %H:%M:%S") {
        return ndt.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    
    // Format 3: "%Y-%m-%d %H:%M:%S" (e.g., "2025-08-11 16:08:39")
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return ndt.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    
    // Format 4: "%d/%m/%Y %H:%M:%S" (e.g., "11/08/2025 16:08:39")
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%d/%m/%Y %H:%M:%S") {
        return ndt.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    
    // Format 5: "%Y/%m/%d %H:%M:%S" (e.g., "2025/08/11 16:08:39")
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y/%m/%d %H:%M:%S") {
        return ndt.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    
    // If no format matches, return the original value
    value.to_string()
}

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = date.to_rfc3339();
    serializer.serialize_str(&formatted)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    let bson_value: bson::Bson = bson::Bson::deserialize(deserializer)?;
    
    match bson_value {
        bson::Bson::DateTime(dt) => Ok(dt.to_chrono()),
        bson::Bson::String(s) => {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%d-%m-%Y %H:%M:%S")
                        .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                })
                .map_err(|e| D::Error::custom(format!("failed to parse datetime: {}", e)))
        }
        _ => Err(D::Error::custom(format!("expected DateTime, got {:?}", bson_value))),
    }
}

// Module for optional DateTime fields
pub mod option {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};
    
    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => {
                let formatted = dt.to_rfc3339();
                serializer.serialize_some(&formatted)
            }
            None => serializer.serialize_none(),
        }
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        
        let opt: Option<bson::Bson> = Option::deserialize(deserializer)?;
        
        match opt {
            Some(bson::Bson::DateTime(dt)) => Ok(Some(dt.to_chrono())),
            Some(bson::Bson::String(s)) => {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| Some(dt.with_timezone(&Utc)))
                    .or_else(|_| {
                        chrono::NaiveDateTime::parse_from_str(&s, "%d-%m-%Y %H:%M:%S")
                            .map(|ndt| Some(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc)))
                    })
                    .map_err(|e| D::Error::custom(format!("failed to parse datetime: {}", e)))
            }
            Some(_) => Err(D::Error::custom("expected DateTime or String")),
            None => Ok(None),
        }
    }
}
