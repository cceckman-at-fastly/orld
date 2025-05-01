use std::io::Write;
use std::time::Duration;

use fastly::cache::core::{insert, lookup, CacheKey};
use fastly::http::StatusCode;
use fastly::{Error, Request, Response};
use uuid::Uuid;

fn new_key() -> CacheKey {
    Uuid::new_v4().into_bytes().to_vec().into()
}

#[fastly::main]
fn main(_req: Request) -> Result<Response, Error> {
    // When we do a full body, then a range request, we get only the ranged bits.
    let key = new_key();

    {
        let fetch = lookup(key.clone())
            .execute()
            .expect("failed initial lookup");
        assert!(fetch.is_none());
    }

    let body = "hello beautiful world".as_bytes();
    {
        let mut writer = insert(key.clone(), Duration::from_secs(10))
            .execute()
            .unwrap();
        writer.write_all(body).unwrap();
        writer.finish().unwrap();
    }

    let got = lookup(key.clone()).execute().unwrap().unwrap();
    let got = got
        .to_stream_from_range(None, Some(4))
        .unwrap()
        .into_bytes();
    let s = std::str::from_utf8(&got).unwrap();
    let status = if s == "hello" {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    Ok(Response::new().with_status(status).with_body_text_plain(s))
}
