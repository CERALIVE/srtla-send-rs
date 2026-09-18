use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::Result;

const OUTPUT_TAIL_BYTES: usize = 64 * 1024;

pub(super) struct Capture {
    bytes: VecDeque<u8>,
    newlines: u64,
    last: Option<u8>,
}

pub(super) type OutputTail = Arc<Mutex<Capture>>;

pub(super) fn new_output_tail() -> OutputTail {
    Arc::new(Mutex::new(Capture {
        bytes: VecDeque::with_capacity(OUTPUT_TAIL_BYTES),
        newlines: 0,
        last: None,
    }))
}

pub(super) fn drain_pipe<R>(mut reader: R, tail: OutputTail) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }
            if let Ok(mut output) = tail.lock() {
                for byte in &buffer[..bytes_read] {
                    output.newlines += u64::from(*byte == b'\n');
                }
                output.last = Some(buffer[bytes_read - 1]);
                output.bytes.extend(&buffer[..bytes_read]);
                while output.bytes.len() > OUTPUT_TAIL_BYTES {
                    output.bytes.pop_front();
                }
            }
        }
    })
}

pub(super) fn output_lines(tail: &OutputTail) -> Vec<String> {
    let Ok(mut output) = tail.lock() else {
        return Vec::new();
    };
    String::from_utf8_lossy(output.bytes.make_contiguous())
        .lines()
        .map(str::to_owned)
        .collect()
}

pub(super) fn line_count(tail: &OutputTail) -> Result<u64> {
    let output = tail
        .lock()
        .map_err(|_| anyhow::anyhow!("poisoned process output capture"))?;
    Ok(output.newlines + u64::from(output.last.is_some_and(|byte| byte != b'\n')))
}
