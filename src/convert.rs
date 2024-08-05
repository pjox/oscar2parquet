use crate::oscar::Document;
use arrow::{
    array::{
        ArrayBuilder, ArrayRef, Float32Builder, Int32Builder, LargeStringBuilder, ListBuilder,
        RecordBatch, StringBuilder, StructArray,
    },
    datatypes::{DataType, Field},
};
//use futures::{stream, StreamExt};
use parquet::{
    arrow::ArrowWriter,
    basic::{Compression, ZstdLevel},
    file::properties::WriterProperties,
};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
};
use tokio::{sync::Semaphore, task::JoinSet};
use walkdir::{DirEntry, WalkDir};

// Converts `Vec<LoteBuilder>` into `StructArray`
#[derive(Debug, Default)]
struct OscarBuilder {
    warc_record_id: StringBuilder,
    warc_refers_to: StringBuilder,
    warc_target_uri: StringBuilder,
    warc_date: StringBuilder,

    content: LargeStringBuilder,

    identified_doc_lang: StringBuilder,
    identified_doc_prob: Float32Builder,

    sentences_langs: ListBuilder<StringBuilder>,
    sentences_probs: ListBuilder<Float32Builder>,

    warc_identified_content_language: ListBuilder<StringBuilder>,

    harmful_pp: Float32Builder,
    tlsh: StringBuilder,
    quality_warnings: ListBuilder<StringBuilder>,
    categories: ListBuilder<StringBuilder>,

    warc_type: StringBuilder,
    content_length: Int32Builder,
    warc_block_digest: StringBuilder,
    content_type: StringBuilder,
}

impl OscarBuilder {
    fn append(&mut self, document: &Document) {
        self.warc_record_id
            .append_option(document.warc_headers.warc_record_id.as_ref());
        self.warc_refers_to
            .append_option(document.warc_headers.warc_refers_to.as_ref());
        self.warc_target_uri
            .append_option(document.warc_headers.warc_target_uri.as_ref());
        self.warc_date
            .append_option(document.warc_headers.warc_date.as_ref());

        self.content.append_value(document.content.as_str());

        self.identified_doc_lang
            .append_value(document.metadata.identification.label.as_str());

        self.identified_doc_prob
            .append_value(document.metadata.identification.prob);

        let mut senteces_langs: Vec<Option<String>> = vec![];
        let mut sentences_probs: Vec<Option<f32>> = vec![];

        for sentence in document.metadata.sentence_identifications.iter() {
            senteces_langs.push(sentence.as_ref().map(|s| s.label.clone()));
            sentences_probs.push(sentence.as_ref().map(|s| s.prob));
        }

        self.sentences_langs.append_value(senteces_langs);
        self.sentences_probs.append_value(sentences_probs);

        let id_langs: Option<Vec<Option<String>>> = document
            .warc_headers
            .warc_identified_content_language
            .as_ref()
            .map(|s| {
                s.split(",")
                    .map(|s| Some(s.to_string()))
                    .collect::<Vec<Option<String>>>()
            });

        self.warc_identified_content_language
            .append_option(id_langs);

        self.harmful_pp.append_option(document.metadata.harmful_pp);
        self.tlsh.append_option(document.metadata.tlsh.as_ref());
        self.quality_warnings
            .append_option(document.metadata.quality_warnings.clone());
        self.categories
            .append_option(document.metadata.categories.clone());

        self.warc_type
            .append_option(document.warc_headers.warc_type.as_ref());

        let length = document
            .warc_headers
            .content_length
            .as_ref()
            .map(|s| s.parse::<i32>().unwrap_or(-1));

        self.content_length.append_option(length);
        self.warc_block_digest
            .append_option(document.warc_headers.warc_block_digest.as_ref());
        self.content_type
            .append_option(document.warc_headers.content_type.as_ref());
    }

    /// Note: returns StructArray to allow nesting within another array if desired
    fn finish(&mut self) -> StructArray {
        let warc_record_id = Arc::new(self.warc_record_id.finish()) as ArrayRef;
        let warc_record_id_field = Arc::new(Field::new("warc_record_id", DataType::Utf8, true));

        let warc_refers_to = Arc::new(self.warc_refers_to.finish()) as ArrayRef;
        let warc_refers_to_field = Arc::new(Field::new("warc_refers_to", DataType::Utf8, true));

        let warc_target_uri = Arc::new(self.warc_target_uri.finish()) as ArrayRef;
        let warc_target_uri_field = Arc::new(Field::new("warc_target_uri", DataType::Utf8, true));

        let warc_date = Arc::new(self.warc_date.finish()) as ArrayRef;
        let warc_date_field = Arc::new(Field::new("warc_date", DataType::Utf8, true));

        let content = Arc::new(self.content.finish()) as ArrayRef;
        let content_field = Arc::new(Field::new("content", DataType::LargeUtf8, false));

        let identified_doc_lang = Arc::new(self.identified_doc_lang.finish()) as ArrayRef;
        let identified_doc_lang_field =
            Arc::new(Field::new("identified_doc_lang", DataType::Utf8, false));

        let identified_doc_prob = Arc::new(self.identified_doc_prob.finish()) as ArrayRef;
        let identified_doc_prob_field =
            Arc::new(Field::new("identified_doc_prob", DataType::Float32, false));

        let sentences_langs = Arc::new(self.sentences_langs.finish()) as ArrayRef;
        let senteces_langs_value_field = Arc::new(Field::new("item", DataType::Utf8, true));
        let sentences_langs_field = Arc::new(Field::new(
            "sentence_langs",
            DataType::List(senteces_langs_value_field),
            true,
        ));

        let sentences_probs = Arc::new(self.sentences_probs.finish()) as ArrayRef;
        let sentences_probs_value_field = Arc::new(Field::new("item", DataType::Float32, true));
        let sentences_probs_field = Arc::new(Field::new(
            "sentences_probs",
            DataType::List(sentences_probs_value_field),
            true,
        ));

        let warc_identified_content_language =
            Arc::new(self.warc_identified_content_language.finish()) as ArrayRef;
        let warc_identified_content_language_value_field =
            Arc::new(Field::new("item", DataType::Utf8, true));
        let warc_identified_content_language_field = Arc::new(Field::new(
            "warc_identified_content_language",
            DataType::List(warc_identified_content_language_value_field),
            true,
        ));

        let harmful_pp = Arc::new(self.harmful_pp.finish()) as ArrayRef;
        let harmful_pp_field = Arc::new(Field::new("harmful_pp", DataType::Float32, true));

        let tlsh = Arc::new(self.tlsh.finish()) as ArrayRef;
        let tlsh_field = Arc::new(Field::new("tlsh", DataType::Utf8, true));

        let quality_warnings = Arc::new(self.quality_warnings.finish()) as ArrayRef;
        let quality_warnings_value_field = Arc::new(Field::new("item", DataType::Utf8, true));
        let quality_warnings_field = Arc::new(Field::new(
            "quality_warnings",
            DataType::List(quality_warnings_value_field),
            true,
        ));

        let categories = Arc::new(self.categories.finish()) as ArrayRef;
        let categories_value_field = Arc::new(Field::new("item", DataType::Utf8, true));
        let categories_field = Arc::new(Field::new(
            "categories",
            DataType::List(categories_value_field),
            true,
        ));

        let warc_type = Arc::new(self.warc_type.finish()) as ArrayRef;
        let warc_type_field = Arc::new(Field::new("warc_type", DataType::Utf8, true));

        let content_length = Arc::new(self.content_length.finish()) as ArrayRef;
        let content_length_field = Arc::new(Field::new("content_length", DataType::Int32, true));

        let warc_block_digest = Arc::new(self.warc_block_digest.finish()) as ArrayRef;
        let warc_block_digest_field =
            Arc::new(Field::new("warc_block_digest", DataType::Utf8, true));

        let content_type = Arc::new(self.content_type.finish()) as ArrayRef;
        let content_type_field = Arc::new(Field::new("content_type", DataType::Utf8, true));

        StructArray::from(vec![
            (warc_record_id_field, warc_record_id),
            (warc_refers_to_field, warc_refers_to),
            (warc_target_uri_field, warc_target_uri),
            (warc_date_field, warc_date),
            (content_field, content),
            (identified_doc_lang_field, identified_doc_lang),
            (identified_doc_prob_field, identified_doc_prob),
            (sentences_langs_field, sentences_langs),
            (sentences_probs_field, sentences_probs),
            (
                warc_identified_content_language_field,
                warc_identified_content_language,
            ),
            (harmful_pp_field, harmful_pp),
            (tlsh_field, tlsh),
            (quality_warnings_field, quality_warnings),
            (categories_field, categories),
            (warc_type_field, warc_type),
            (content_length_field, content_length),
            (warc_block_digest_field, warc_block_digest),
            (content_type_field, content_type),
        ])
    }
}

impl<'a> Extend<&'a Document> for OscarBuilder {
    fn extend<T: IntoIterator<Item = &'a Document>>(&mut self, iter: T) {
        iter.into_iter().for_each(|row| self.append(row));
    }
}

/// Converts a slice of [`Document`] to a [`RecordBatch`]
// fn rows_to_batch(rows: &[Document]) -> RecordBatch {
//     let mut builder = OscarBuilder::default();
//     builder.extend(rows);
//     RecordBatch::from(&builder.finish())
// }

fn write_to_parquet(batch: RecordBatch, folder_path: &PathBuf, lang: &str, part: usize) {
    let mut path = folder_path.clone();
    path.push(format!("{}_part_{}.parquet", lang, part));
    let parquet = File::create(path).unwrap();

    let properties = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap()))
        .build();

    let mut writer = ArrowWriter::try_new(parquet, batch.schema(), Some(properties)).unwrap();
    writer.write(&batch).expect("Writing batch");
    writer.close().unwrap();
}

async fn process_file(file: DirEntry, dst: PathBuf) {
    // Create the output folder if it does not exist
    let mut folder_path = PathBuf::new();
    folder_path.push(dst);
    let lang = file
        .file_name()
        .to_str()
        .unwrap()
        .strip_suffix("_meta.jsonl")
        .unwrap();
    folder_path.push(lang);

    if !folder_path.exists() {
        std::fs::create_dir_all(&folder_path).unwrap();
    }

    // let mut records: Vec<Document> = vec![];
    let mut record_builder = OscarBuilder::default();

    let jsonl = {
        let file = File::open(file.path()).unwrap();
        BufReader::new(file)
    };

    println!("Processing file: {}", file.file_name().to_str().unwrap());

    let mut part = 1;

    for line in jsonl.lines() {
        let line = line.unwrap();
        let document: Document = serde_json::from_str(&line).unwrap();
        record_builder.append(&document);
        if record_builder.warc_record_id.len() >= 90_000 {
            write_to_parquet(
                RecordBatch::from(&record_builder.finish()),
                &folder_path,
                lang,
                part,
            );
            record_builder = OscarBuilder::default();
            part += 1;
        }
    }
    write_to_parquet(
        RecordBatch::from(&record_builder.finish()),
        &folder_path,
        lang,
        part,
    );
    println!(
        "Finished processing file: {}",
        file.file_name().to_str().unwrap(),
    );
}

pub async fn convert_to_parquet(src: &PathBuf, dst: &PathBuf, threads: usize) {
    let file_paths: Vec<DirEntry> = WalkDir::new(src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.file_name().to_str().unwrap().ends_with(".jsonl"))
        .collect();

    let semaphore = Arc::new(Semaphore::new(threads));
    let mut set = JoinSet::new();

    for file in file_paths {
        let dst = dst.clone();
        let semaphore = semaphore.clone();
        set.spawn(async move {
            let _permit = semaphore.acquire().await;
            process_file(file, dst).await;
        });
    }

    while let Some(result) = set.join_next().await {
        result.unwrap();
    }
}
