#![warn(missing_debug_implementations, rust_2018_idioms)]
#![feature(async_await)]

use futures::executor;
use futures::{AsyncReadExt, AsyncWriteExt};

use merge_io::MergeIO;
use std::io::{Cursor, Result};

#[test]
fn test_duplex() -> Result<()> {
    executor::block_on(async {
        let read_sample = vec![1, 2, 3, 4];
        let write_sample = vec![10, 20, 30, 40];

        let reader = Cursor::new(read_sample.clone());
        let writer = Cursor::new(vec![0u8; 1024]);
        let mut tio = MergeIO::new(reader, writer);

        tio.write_all(&write_sample).await?;

        let mut read_buf = Vec::<u8>::with_capacity(1024);
        tio.read_to_end(&mut read_buf).await?;

        assert_eq!(&read_buf, &read_sample);

        let (outcome_read_buf, outcome_write_buf) = tio.into_inner();

        assert_eq!(outcome_read_buf.position(), read_sample.len() as u64);
        assert_eq!(
            outcome_write_buf.into_inner()[..write_sample.len()],
            write_sample[..]
        );

        Ok(())
    })
}
