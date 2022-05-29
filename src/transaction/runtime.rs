//! Transaction processor.
//!

use std::fs::File;
use std::io;
use std::path::Path;
use std::thread;

use crossbeam::channel;

use super::{Reader, Writer};
use crate::error::Error;
use crate::transport::Sender;
use crate::Result;

/// Run everything.
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
    let w_handle = thread::spawn(move || writer.write());

    if let Err(err) = r_handle.join() {
        tracing::error!(err.cause_chain=?err);
    }
    if let Err(err) = w_handle.join() {
        tracing::error!(err.cause_chain=?err);
    }
}

/// Creates an io::Reader from file path.
fn new_reader(path: impl AsRef<Path>) -> Result<impl io::Read> {
    File::open(path).map_err(Error::IoError)
}

/// Gets input file.

pub fn get_input() -> Result<impl io::Read> {
    let path = std::env::args().nth(1).ok_or(Error::InvalidArgumentError)?;
    new_reader(path)
}
