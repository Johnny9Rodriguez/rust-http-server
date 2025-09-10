#![allow(dead_code, unused_variables)]

use std::io::{self, Error, Read};

pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

enum RequestState {
    Initialized,
    Done,
}

pub struct Request {
    pub request_line: Option<RequestLine>,
    state: RequestState,
}

impl Request {
    fn new() -> Self {
        Self {
            request_line: None,
            state: RequestState::Initialized,
        }
    }

    fn parse(&mut self, data: &str) -> Result<usize, io::Error> {
        match self.state {
            RequestState::Initialized => {
                let (consumed, maybe_line) = parse_request_line(data)?;
                if let Some(line) = maybe_line {
                    self.request_line = Some(line);
                    self.state = RequestState::Done;
                }
                Ok(consumed)
            }
            RequestState::Done => Ok(0),
        }
    }
}

pub fn request_from_reader<R: Read>(mut r: R) -> Result<Request, std::io::Error> {
    let mut req = Request::new();
    let mut buf = Vec::with_capacity(8);
    let mut tmp = [0u8; 8];

    loop {
        let n = r.read(&mut tmp)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF before request complete",
            ));
        }

        buf.extend_from_slice(&tmp[..n]);

        let s = std::str::from_utf8(&buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))?;

        let consumed = req.parse(s)?;
        if consumed > 0 {
            buf.drain(..consumed);
        }

        if let RequestState::Done = req.state {
            return Ok(req);
        }
    }
}

fn parse_request_line(s: &str) -> Result<(usize, Option<RequestLine>), io::Error> {
    if let Some(n) = s.find("\r\n") {
        let data = &s[..n];
        let mut parts = data.split_whitespace();

        let method = parts
            .next()
            .ok_or_else(|| Error::new(io::ErrorKind::InvalidData, "missing method"))?;

        let method = match method {
            "GET" => "GET".to_string(),
            "POST" => "POST".to_string(),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported method",
                ));
            }
        };

        let request_target = parts
            .next()
            .ok_or_else(|| Error::new(io::ErrorKind::InvalidData, "missing request target"))?
            .to_string();

        let http_version = parts
            .next()
            .and_then(|s| s.strip_prefix("HTTP/"))
            .ok_or_else(|| {
                Error::new(
                    io::ErrorKind::InvalidData,
                    "missing or invalid http version",
                )
            })?
            .to_string();

        if let Some(_) = parts.next() {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                "too many parts in request line",
            ));
        }

        return Ok((
            n,
            Some(RequestLine {
                http_version,
                request_target,
                method,
            }),
        ));
    }

    Ok((0, None))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        chunk_reader::ChunkReader,
        request::{RequestState, request_from_reader},
    };

    #[test]
    fn test_good_get_request_line() {
        let reader = ChunkReader::new(
            "GET / HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
            1,
        );

        let result = request_from_reader(reader);

        assert!(result.is_ok());

        let r = result.unwrap();
        assert!(matches!(r.state, RequestState::Done));

        let line = r.request_line.expect("request line should be parsed");

        assert_eq!(line.method, "GET");
        assert_eq!(line.request_target, "/");
        assert_eq!(line.http_version, "1.1");
    }

    #[test]
    fn test_good_get_request_line_with_path() {
        let reader = ChunkReader::new(
            "GET /coffee HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
            3,
        );

        let result = request_from_reader(reader);

        assert!(result.is_ok());

        let r = result.unwrap();
        assert!(matches!(r.state, RequestState::Done));

        let line = r.request_line.expect("request line should be parsed");

        assert_eq!(line.method, "GET");
        assert_eq!(line.request_target, "/coffee");
        assert_eq!(line.http_version, "1.1");
    }

    #[test]
    fn test_invalid_number_of_parts_in_request_line() {
        let input = Cursor::new(
            "/coffee HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
        );

        let result = request_from_reader(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_good_post_request_line() {
        let input = Cursor::new(
            "POST / HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
        );

        let result = request_from_reader(input);

        assert!(result.is_ok());

        let r = result.unwrap();
        assert!(matches!(r.state, RequestState::Done));

        let line = r.request_line.expect("request line should be parsed");

        assert_eq!(line.method, "POST");
        assert_eq!(line.request_target, "/");
        assert_eq!(line.http_version, "1.1");
    }

    #[test]
    fn test_invalid_out_of_order_request_line() {
        let input = Cursor::new(
            "/ GET HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
        );

        let result = request_from_reader(input);

        assert!(result.is_err());
    }
}
