use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Document {
    pub content: String,
    pub warc_headers: WarcHeaders,
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize)]
pub struct Identification {
    pub label: String,
    pub prob: f32,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub identification: Identification,
    pub harmful_pp: Option<f32>,
    pub tlsh: Option<String>,
    pub quality_warnings: Option<Vec<Option<String>>>,
    pub categories: Option<Vec<Option<String>>>,
    pub sentence_identifications: Vec<Option<Identification>>,
}

#[derive(Debug, Deserialize)]
pub struct WarcHeaders {
    #[serde(rename = "warc-identified-content-language")]
    pub warc_identified_content_language: Option<String>,
    #[serde(rename = "warc-target-uri")]
    pub warc_target_uri: Option<String>,
    #[serde(rename = "warc-record-id")]
    pub warc_record_id: Option<String>,
    #[serde(rename = "warc-type")]
    pub warc_type: Option<String>,
    #[serde(rename = "content-length")]
    pub content_length: Option<String>,
    #[serde(rename = "warc-refers-to")]
    pub warc_refers_to: Option<String>,
    #[serde(rename = "warc-block-digest")]
    pub warc_block_digest: Option<String>,
    #[serde(rename = "warc-date")]
    pub warc_date: Option<String>,
    #[serde(rename = "content-type")]
    pub content_type: Option<String>,
}
