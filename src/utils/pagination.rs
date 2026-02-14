use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    50
}

fn deserialize_i64_from_str_or_number<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrI64 {
        Str(String),
        Num(i64),
    }

    match StrOrI64::deserialize(deserializer)? {
        StrOrI64::Num(n) => Ok(n),
        StrOrI64::Str(s) => s
            .parse::<i64>()
            .map_err(|e| de::Error::custom(format!("invalid i64 '{}': {}", s, e))),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page", rename = "currentPage")]
    #[serde(deserialize_with = "deserialize_i64_from_str_or_number")]
    pub currentPage: i64,
    #[serde(default = "default_limit", rename = "itemsPerPage")]
    #[serde(deserialize_with = "deserialize_i64_from_str_or_number")]
    pub itemsPerPage: i64,
    #[serde(rename = "orderField")]
    pub order_field: Option<String>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    #[serde(rename = "lastPage")]
    pub last_page: i64,
    #[serde(rename = "currentPage")]
    pub current_page: i64,
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: i64,
    pub prev: i64,
    pub next: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub message: String,
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

impl<T> PaginationResponse<T> {
    pub fn new(message: impl Into<String>, data: Vec<T>, total: i64, current_page: i64, per_page: i64) -> Self {
        let last_page = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as i64
        } else {
            0
        };

        let prev = if current_page > 1 {
            current_page - 1
        } else {
            0
        };

        let next = if current_page < last_page {
            current_page + 1
        } else {
            0
        };

        Self {
            message: message.into(),
            data,
            meta: PaginationMeta {
                total,
                last_page,
                current_page,
                items_per_page: per_page,
                prev,
                next,
            },
        }
    }
}
