use automerge::{AutoCommit, ObjId, ObjType};
use automerge::transaction::Transactable;
use std::fs::File;
use std::io::{BufReader, Read};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TestPatch(pub usize, pub usize, pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct TestTxn {
    pub patches: Vec<TestPatch>
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestData {
    #[serde(rename = "startContent")]
    pub start_content: String,
    #[serde(rename = "endContent")]
    pub end_content: String,

    pub txns: Vec<TestTxn>,
}

impl TestData {
    pub fn len(&self) -> usize {
        self.txns.iter()
            .map(|txn| { txn.patches.len() })
            .sum::<usize>()
    }

    pub fn is_empty(&self) -> bool {
        !self.txns.iter().any(|txn| !txn.patches.is_empty())
    }
}

// TODO: Make a try_ version of this method, which returns an appropriate Error object.
pub fn load_testing_data(filename: &str) -> TestData {
    let file = File::open(filename).unwrap();

    let mut reader = BufReader::new(file);
    let mut raw_json = vec!();
    reader.read_to_end(&mut raw_json).unwrap();

    let data: TestData = serde_json::from_reader(raw_json.as_slice()).unwrap();

    data
}

fn apply_edits_local_am(doc: &mut AutoCommit, id: &ObjId, txns: &Vec<TestTxn>) {
    for (_i, txn) in txns.iter().enumerate() {
        // if _i % 1000 == 0 { println!("{_i}"); }
        for TestPatch(pos, del_span, ins_content) in &txn.patches {
            doc.splice_text(id, *pos, *del_span, ins_content).unwrap();
        }
    }
}

fn main() {
    let name = "rustcode.json";
    let mut doc = AutoCommit::new();
    let text = doc.put_object(&automerge::ROOT, "doc", ObjType::Text).unwrap();
    let data = load_testing_data(name);

    let start = std::time::Instant::now();

    println!("Processing {} edits...", data.len());
    apply_edits_local_am(&mut doc, &text, &data.txns);
    let elapsed_time = start.elapsed();
    println!("Processing took {:?}", elapsed_time);

    assert_eq!(&doc.text(&text).unwrap(), &data.end_content);
}
