use std::{
    fs::File,
    io::{BufReader, Read, Result},
};

fn main() -> Result<()> {
    let file = File::open("messages.txt")?;
    let mut buf_reader = BufReader::new(file);

    loop {
        let mut s = String::new();
        let bytes_read = buf_reader.by_ref().take(8).read_to_string(&mut s)?;

        if bytes_read == 0 {
            break;
        }

        println!("read: {s}")
    }

    Ok(())
}
