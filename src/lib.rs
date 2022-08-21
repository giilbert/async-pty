#[cfg(test)]
mod tests;

use futures_util::{Sink, Stream};
use std::{
    error::Error,
    ffi::OsStr,
    io::{Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

pub struct PtySink(Box<dyn Write + Send>, Box<dyn MasterPty + Send>);
pub enum PtyInput<'a> {
    Text(&'a str),
    Resize(u16, u16),
}

impl Sink<PtyInput<'_>> for PtySink {
    type Error = Box<dyn Error>;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: PtyInput) -> Result<(), Self::Error> {
        match item {
            PtyInput::Text(data) => {
                self.0.write(data.as_bytes())?;
            }
            PtyInput::Resize(rows, cols) => {
                self.1.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })?;
            }
        };
        Ok(())
    }

    // figure these two functions out!
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

pub struct PtyStream(Box<dyn Read + Send>, Box<dyn Child + Send + Sync>);

impl Stream for PtyStream {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Ok(Some(_status)) = self.1.try_wait() {
            return Poll::Ready(None);
        }

        let mut buffer = [0; 1024];
        if let Ok(length) = self.0.read(&mut buffer) {
            return Poll::Ready(Some(
                String::from_utf8_lossy(&buffer[0..length])
                    .parse::<String>()
                    .unwrap(),
            ));
        }

        Poll::Pending
    }
}

pub fn create<S: AsRef<OsStr>>(
    command: S,
    default_size: PtySize,
) -> Result<(PtySink, PtyStream), anyhow::Error> {
    let pty_system = native_pty_system();

    let pair = pty_system.openpty(default_size)?;

    let cmd = CommandBuilder::new(command);
    let child = pair.slave.spawn_command(cmd)?;

    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.try_clone_writer()?;

    let stream = PtyStream(reader, child);
    let sink = PtySink(writer, pair.master);

    Ok((sink, stream))
}
