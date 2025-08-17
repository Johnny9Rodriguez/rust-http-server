#![allow(dead_code, unused_variables)]

use std::io::{self, Error, Read};

struct Request {
    request_line: RequestLine,
}

struct RequestLine {
    http_version: String,
    request_target: String,
    method: String,
}

fn request_from_reader<R: Read>(mut r: R) -> Result<Request, std::io::Error> {
    let mut s = String::new();
    r.read_to_string(&mut s)?;

    let request_line = parse_request_line(&s)?;

    Ok(Request { request_line })
}

fn parse_request_line(s: &str) -> Result<RequestLine, std::io::Error> {
    let raw_data = s
        .splitn(2, "\r\n")
        .next()
        .ok_or_else(|| Error::new(io::ErrorKind::InvalidData, "missing request line"))?;

    let mut parts = raw_data.split_whitespace();

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

    Ok(RequestLine {
        method,
        request_target,
        http_version,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::request::request_from_reader;

    #[test]
    fn test_good_get_request_line() {
        let input = Cursor::new(
            "GET / HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
        );

        let result = request_from_reader(input);

        assert!(result.is_ok());

        let r = result.unwrap();

        assert_eq!(r.request_line.method, "GET");
        assert_eq!(r.request_line.request_target, "/");
        assert_eq!(r.request_line.http_version, "1.1");
    }

    #[test]
    fn test_good_get_request_line_with_path() {
        let input = Cursor::new(
            "GET /coffee HTTP/1.1\r\n
            Host: localhost:42069\r\n
            User-Agent: curl/7.81.0\r\n
            Accept: */*\r\n\r\n",
        );

        let result = request_from_reader(input);

        assert!(result.is_ok());

        let r = result.unwrap();

        assert_eq!(r.request_line.method, "GET");
        assert_eq!(r.request_line.request_target, "/coffee");
        assert_eq!(r.request_line.http_version, "1.1");
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

        assert_eq!(r.request_line.method, "POST");
        assert_eq!(r.request_line.request_target, "/");
        assert_eq!(r.request_line.http_version, "1.1");
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
