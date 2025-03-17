
se atrium_api::types::Unknown;

// @generated - This file is generated by esquema-codegen (forked from atrium-codegen). DO NOT EDIT.
// !A collection of known record types.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum KnownRecord {
    #[serde(rename = "xyz.statusphere.status")]
    LexiconsXyzStatusphereStatus(Box<crate::lexicons::xyz::statusphere::status::Record>),
}
impl From<crate::lexicons::xyz::statusphere::status::Record> for KnownRecord {
    fn from(record: crate::lexicons::xyz::statusphere::status::Record) -> Self {
        KnownRecord::LexiconsXyzStatusphereStatus(Box::new(record))
    }
}
impl From<crate::lexicons::xyz::statusphere::status::RecordData> for KnownRecord {
    fn from(record_data: crate::lexicons::xyz::statusphere::status::RecordData) -> Self {
        KnownRecord::LexiconsXyzStatusphereStatus(Box::new(record_data.into()))
    }
}

impl Into<atrium_api::types::Unknown> for KnownRecord {
    fn into(self) -> atrium_api::types::Unknown {
        atrium_api::types::TryIntoUnknown::try_into_unknown(&self).unwrap()
    }
}

