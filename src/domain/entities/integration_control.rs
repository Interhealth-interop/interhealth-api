use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use bson::Bson;
use chrono::{DateTime, Utc};
use crate::utils::utils::object_id_format;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationControl {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        skip_serializing_if = "Option::is_none",
        with = "object_id_format"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "databaseViewId")]
    pub database_view_id: String,
    #[serde(rename = "cron")]
    pub cron: String,
    #[serde(rename = "dateField")]
    pub date_field: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "startAt",
        deserialize_with = "deserialize_optional_bson_datetime"
    )]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "endAt",
        deserialize_with = "deserialize_optional_bson_datetime"
    )]
    pub end_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "lastRunAt",
        deserialize_with = "deserialize_optional_bson_datetime"
    )]
    pub last_run_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: String,
    pub company_id: String,
    #[serde(with = "crate::utils::utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "crate::utils::utils::date_format")]
    pub updated_at: DateTime<Utc>,
}

fn deserialize_optional_bson_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as DeError;

    let value: Option<Bson> = Option::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Bson::DateTime(dt) => Ok(Some(dt.to_chrono())),
        Bson::String(s) => chrono::DateTime::parse_from_rfc3339(&s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(|_| D::Error::custom("Invalid datetime format")),
        Bson::Document(doc) => {
            if let Some(date_value) = doc.get("$date") {
                match date_value {
                    Bson::String(s) => chrono::DateTime::parse_from_rfc3339(s)
                        .map(|dt| Some(dt.with_timezone(&Utc)))
                        .map_err(|_| D::Error::custom("Invalid datetime format")),
                    Bson::Int64(ms) => Ok(Some(DateTime::<Utc>::from_timestamp_millis(*ms).ok_or_else(|| {
                        D::Error::custom("Invalid datetime format")
                    })?)),
                    Bson::Int32(ms) => Ok(Some(DateTime::<Utc>::from_timestamp_millis(i64::from(*ms)).ok_or_else(|| {
                        D::Error::custom("Invalid datetime format")
                    })?)),
                    Bson::Document(inner) => {
                        if let Some(Bson::String(num)) = inner.get("$numberLong") {
                            let ms = num
                                .parse::<i64>()
                                .map_err(|_| D::Error::custom("Invalid datetime format"))?;
                            Ok(Some(DateTime::<Utc>::from_timestamp_millis(ms).ok_or_else(|| {
                                D::Error::custom("Invalid datetime format")
                            })?))
                        } else {
                            Err(D::Error::custom("Invalid datetime format"))
                        }
                    }
                    _ => Err(D::Error::custom("Invalid datetime format")),
                }
            } else {
                Err(D::Error::custom("Invalid datetime format"))
            }
        }
        _ => Err(D::Error::custom("Invalid datetime format")),
    }
}
