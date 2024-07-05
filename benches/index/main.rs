#![feature(test)]

use test::Bencher;

extern crate test;
extern crate rucene;

use rucene::core::analysis::WhitespaceTokenizer;
use rucene::core::doc::{Field, FieldType, Fieldable, IndexOptions, NumericDocValuesField, Term};
use rucene::core::index::reader::IndexReader;
use rucene::core::index::writer::{IndexWriter, IndexWriterConfig};
use rucene::core::search::collector::TopDocsCollector;
use rucene::core::search::query::TermQuery;
use rucene::core::search::{DefaultIndexSearcher, IndexSearcher};
use rucene::core::store::directory::FSDirectory;

use std::fs::File;
use std::io::{self, BufRead, BufReader};
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

impl io::Read for StringReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remain = buf.len().min(self.text.len() - self.index);
        if remain > 0 {
            buf[..remain].copy_from_slice(&self.text.as_bytes()[self.index..self.index + remain]);
            self.index += remain;
        }
        Ok(remain)
    }
}

#[bench]
fn index(b: &mut Bencher) -> Result<()> {
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

    let fpath = "corpus-webis-tldr-17.json";
    b.iter( || {
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
    });

    // flush to disk
    writer.commit()?;

    // new index search
    let reader = writer.get_reader(true, false)?;
    let index_searcher = DefaultIndexSearcher::new(Arc::new(reader), None);

    // search
    let query: TermQuery = TermQuery::new(
        Term::new("title".into(), "fox".as_bytes().to_vec()),
        1.0,
        None,
    );
    let mut collector = TopDocsCollector::new(10);
    index_searcher.search(&query, &mut collector)?;

    let mut hightlighter = FastVectorHighlighter::default();
    let mut field_query = FieldQuery::new(&query, Some(index_searcher.reader()), false, true)?;
    let top_docs = collector.top_docs();
    println!("total hits: {}", top_docs.total_hits());
    Ok(())
}
