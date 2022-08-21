use crate::{create, PtyInput};

#[cfg(unix)]
#[tokio::test]
async fn input_and_output() {
    use futures_util::{SinkExt, StreamExt};

    let (mut tx, mut rx) = create(
        "cat",
        portable_pty::PtySize {
            rows: 30,
            cols: 40,
            pixel_width: 0,
            pixel_height: 0,
        },
    )
    .unwrap();

    let mut out = String::new();
    tx.send(PtyInput::Text("Hello world\n")).await.unwrap();

    // wait a little bit for the program to echo out the message
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    if let Some(data) = rx.next().await {
        out += &data;
    }

    assert_eq!(out, "Hello world\r\nHello world\r\n");
}
