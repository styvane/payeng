use parking_lot::Mutex;
use payeng::prelude::{runtime, Client};
use rust_decimal::Decimal;

use std::sync::Arc;

struct TestWriter {
    content: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.content.lock().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.content.lock().flush()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Record {
    pub client: Client,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

#[test]
fn run_output_expected_value() {
    let dir = std::env::current_dir().expect("failed to get current directory");
    let path = dir.join("tests/test.csv");
    let reader = std::fs::File::open(path).unwrap();
    let content = Arc::new(Mutex::new(vec![]));
    let writer = TestWriter {
        content: content.clone(),
    };
    runtime::run(reader, writer, 20);
    let content = content.lock().clone();
    let content = String::from_utf8(content).expect("failed to convert to string");

    let mut records = vec![];
    for result in csv::Reader::from_reader(content.as_bytes()).deserialize() {
        let record: Record = result.expect("failed to get record");
        records.push(record);
    }

    records.sort_by_key(|v| v.client.clone());
    insta::assert_csv_snapshot!(records);
}
