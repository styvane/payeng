//! Transaction processor.
//!

use std::io;
use std::thread;

use crossbeam::channel;

use super::{Reader, Writer};
use crate::transport::Sender;

#[tracing::instrument(name = "Run all", skip(reader, writer, capacity))]
pub fn run(
    reader: impl io::Read + Send + 'static,
    writer: impl io::Write + Send + 'static,
    capacity: usize,
) {
    let (outgoing, incoming) = channel::bounded(capacity);
    let mut reader = Reader::from_reader(reader, outgoing);
    let mut writer = Writer::from_writer(writer, incoming);

    let r_handle = thread::spawn(move || reader.send());
    let w_handle = thread::spawn(move || writer.process_transaction());

    r_handle.join();
    w_handle.join();
}
