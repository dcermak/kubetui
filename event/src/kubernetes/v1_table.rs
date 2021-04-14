use k8s_openapi::apimachinery::pkg::apis::meta::v1::ListMeta;
use k8s_openapi::apimachinery::pkg::runtime::RawExtension;
use kube::api::TypeMeta;
use serde::Deserialize;

use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    #[serde(flatten)]
    pub type_meta: TypeMeta,
    pub list_meta: Option<ListMeta>,
    pub column_definitions: Vec<TableColumnDefinition>,
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnDefinition {
    pub name: String,
    pub r#type: String,
    pub format: String,
    pub description: String,
    pub priority: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    pub cells: Vec<Value>,
    pub conditions: Option<Vec<TableRowCondition>>,
    pub object: Option<RawExtension>,
}

pub type RowConditionType = String;
pub type ConditionStatus = String;

#[allow(dead_code)]
pub const ROW_COMPLETED: &str = "Completed";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRowCondition {
    pub r#type: RowConditionType,
    pub status: ConditionStatus,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[allow(dead_code)]
pub const CONDITION_TRUE: &str = "True";
#[allow(dead_code)]
pub const CONDITION_FALSE: &str = "False";
#[allow(dead_code)]
pub const CONDITION_UNKNOWN: &str = "Unknown";
