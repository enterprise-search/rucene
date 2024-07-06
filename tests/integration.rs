#![feature(test)]

use rucene::core::codec::CodecEnum;

extern crate rucene;
extern crate test;

use rucene::core::analysis::WhitespaceTokenizer;
use rucene::core::doc::{Field, FieldType, Fieldable, IndexOptions, NumericDocValuesField};
use rucene::core::index::reader::IndexReader;
use rucene::core::index::writer::{IndexWriter, IndexWriterConfig};
use rucene::core::search::collector::TopDocsCollector;
use rucene::core::search::query::{Query, QueryStringQueryBuilder};
use rucene::core::search::{DefaultIndexSearcher, IndexSearcher};
use rucene::core::store::directory::FSDirectory;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;

use rucene::core::highlight::FastVectorHighlighter;
use rucene::core::highlight::FieldQuery;
use rucene::core::util::VariantValue;
use rucene::error::Result;

fn indexed_text_field_type() -> FieldType {
    let mut field_type = FieldType::default();
    field_type.index_options = IndexOptions::DocsAndFreqsAndPositionsAndOffsets;
    field_type.store_term_vectors = true;
    field_type.store_term_vector_offsets = true;
    field_type.store_term_vector_positions = true;
    field_type
}

fn new_index_text_field(field_name: String, text: String) -> Field {
    let token_stream = WhitespaceTokenizer::new(Box::new(StringReader::new(text)));
    Field::new(
        field_name,
        indexed_text_field_type(),
        None,
        Some(Box::new(token_stream)),
    )
}

fn new_stored_text_field(field_name: String, text: String) -> Field {
    let mut field_type = FieldType::default();
    field_type.stored = true;

    Field::new(
        field_name,
        field_type,
        Some(VariantValue::VString(text)),
        None,
    )
}

struct StringReader {
    text: String,
    index: usize,
}

impl StringReader {
    fn new(text: String) -> Self {
        StringReader { text, index: 0 }
    }
}

impl std::io::Read for StringReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remain = buf.len().min(self.text.len() - self.index);
        if remain > 0 {
            buf[..remain].copy_from_slice(&self.text.as_bytes()[self.index..self.index + remain]);
            self.index += remain;
        }
        Ok(remain)
    }
}

#[test]
fn integration() -> Result<()> {
    // create index directory
    let path = "/tmp/test_rucene";
    let dir_path = Path::new(path);
    if dir_path.exists() {
        std::fs::remove_dir_all(&dir_path)?;
        std::fs::create_dir(&dir_path)?;
    }

    // create index writer
    let config = Arc::new(IndexWriterConfig::default());
    let directory = Arc::new(FSDirectory::with_path(&dir_path)?);
    let writer = IndexWriter::new(directory, config)?;

    let fpath = "tests/fixtures/alice.txt";
    let f = File::open(fpath).expect("failed to open input file");
    for line in BufReader::new(f).lines() {
        let text = line.expect("failed to read line");
        let mut doc: Vec<Box<dyn Fieldable>> = vec![];
        // add indexed text field
        let text_field = new_index_text_field("title".into(), text.clone());
        doc.push(Box::new(text_field));
        // add raw text field, this used for highlight
        let stored_text_field = new_stored_text_field("title.raw".into(), text);
        doc.push(Box::new(stored_text_field));
        // add numeric doc value field
        doc.push(Box::new(NumericDocValuesField::new("weight".into(), 1)));

        // add the document
        writer.add_document(doc).expect("failed to add document");
    }

    // flush to disk
    writer.commit()?;

    // new index search
    let reader = writer.get_reader(true, false)?;
    let index_searcher = DefaultIndexSearcher::new(Arc::new(reader), None);

    // search
    let query_string = "the +dream of +Wonderland";
    let field = "title";
    let query: Box<dyn Query<CodecEnum>> =
        QueryStringQueryBuilder::new(query_string.into(), vec![(field.into(), 1.0)], 0, 1.0)
            .build()
            .unwrap();
    let mut collector = TopDocsCollector::new(10);
    index_searcher.search(query.as_ref(), &mut collector)?;

    let mut hightlighter = FastVectorHighlighter::default();
    let mut field_query =
        FieldQuery::new(query.as_ref(), Some(index_searcher.reader()), false, true)?;
    let top_docs = collector.top_docs();
    assert_eq!(top_docs.total_hits(), 1);
    assert_eq!(top_docs.total_groups(), 1);
    let hit = &top_docs.score_docs()[0];

    let doc_id = hit.doc_id();
    assert_eq!(doc_id, 3400);
    // fetch stored fields
    let stored_fields = vec!["title.raw".into()];
    let stored_doc = index_searcher.reader().document(doc_id, &stored_fields)?;
    assert_eq!(stored_doc.fields.len(), 1);
    let s = &stored_doc.fields[0];
    assert_eq!(s.field.name(), "title.raw");
    assert_eq!(
        *s.field.field_data().unwrap(),
        VariantValue::VString(
            "perhaps even with the dream of Wonderland of long ago: and how she".into()
        )
    );

    // visit doc values
    let leaf = index_searcher.reader().leaf_reader_for_doc(doc_id);
    let doc_values = leaf.reader.get_numeric_doc_values("weight")?;
    assert_eq!(doc_values.get(doc_id)?, 1);

    // highlight
    let highlight_res = hightlighter.get_best_fragments(
        &mut field_query,
        &leaf,
        doc_id,
        "title",
        "title.raw",
        100,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
    )?;
    assert_eq!(highlight_res.len(), 1);
    assert_eq!(
        &highlight_res[0],
        "perhaps even with <b>the</b> <b>dream</b> <b>of</b> <b>Wonderland</b> <b>of</b> long \
         ago: and how she"
    );
    Ok(())
}
