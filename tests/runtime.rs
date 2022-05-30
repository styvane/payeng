use parking_lot::Mutex;
use payeng::prelude::runtime;
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
    let mut settings = insta::Settings::clone_current();
    settings.set_sort_maps(true);
    settings.bind(|| {
        insta::assert_csv_snapshot!(
            String::from_utf8(content).expect("failed to convert to string")
        );
    });
}
