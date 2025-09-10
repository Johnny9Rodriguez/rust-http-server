use std::{io::Result, net::TcpListener};

use rust_http::request;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:42069")?;
    println!("Listening on port 42069");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted connection");

                match request::request_from_reader(stream) {
                    Ok(req) => {
                        if let Some(line) = req.request_line {
                            println!("Request line:");
                            println!("- Method: {}", line.method);
                            println!("- Target: {}", line.request_target);
                            println!("- Version: {}", line.http_version);
                        }
                    }
                    Err(err) => eprintln!("Failed to parse request: {err}"),
                }

                println!("Closed connection");
            }
            Err(err) => eprintln!("Connection error: {err}"),
        }
    }

    Ok(())
}
