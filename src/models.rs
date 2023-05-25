use serde::{Deserialize, Serialize};

/// User details.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(table_name = visitors)]
pub struct Visitors {
    pub id: String,
    pub view_count: i32,
}
