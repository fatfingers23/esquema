// @generated - This file is generated by esquema-codegen (forked from atrium-codegen). DO NOT EDIT.
//!Definitions for the `dev.baileytownsend.weeklySteps` namespace.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordData {
    pub created_at: atrium_api::types::string::Datetime,
    pub steps: i64,
}
pub type Record = atrium_api::types::Object<RecordData>;
