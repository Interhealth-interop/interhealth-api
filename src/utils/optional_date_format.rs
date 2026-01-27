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
        None => Ok(None),
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
        Some(other) => Err(D::Error::custom(format!("expected DateTime or null, got {:?}", other))),
    }
}
