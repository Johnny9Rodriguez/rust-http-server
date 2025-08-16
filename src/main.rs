use std::{
    io::{Read, Result},
    net::TcpListener,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

fn get_lines_channel<T>(mut r: T) -> Receiver<String>
where
    T: Read + Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut buf = [0u8; 8];
        let mut current_line = String::new();

        while let Ok(n) = r.read(&mut buf) {
            if n == 0 {
                break;
            }

            let chunk = String::from_utf8_lossy(&buf[..n]);
            let parts: Vec<&str> = chunk.split('\n').collect();
            let mut iter = parts.into_iter().peekable();

            while let Some(part) = iter.next() {
                current_line.push_str(part);

                if iter.peek().is_some() {
                    tx.send(current_line.clone()).unwrap();
                    current_line.clear();
                }
            }

            // Artifical delay
            thread::sleep(Duration::from_millis(50));
        }
    });

    rx
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:42069")?;

    println!("Listening on port 42069");

    for stream in listener.incoming() {
        println!("Accepted connection");

        let rx = get_lines_channel(stream.unwrap());

        for msg in rx {
            println!("read: {msg}");
        }

        println!("Closed connection");
    }

    Ok(())
}
