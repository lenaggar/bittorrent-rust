use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    if encoded_value.starts_with(|c: char| c.is_digit(10)) {
        return encoded_value
            .split_once(':')
            .and_then(|(text_len, remaining)| {
                let number = text_len.parse::<usize>().ok()?;
                Some((
                    serde_json::Value::String(remaining[..number].to_string()),
                    &remaining[number..],
                ))
            })
            .unwrap();
    } else if encoded_value.starts_with('i') {
        return encoded_value
            .split_once('e')
            .and_then(|(stringified_number, remaining)| {
                Some((
                    stringified_number[1..].parse::<isize>().ok()?.into(),
                    remaining,
                ))
            })
            .unwrap();
    } else if encoded_value.starts_with('l') {
        let mut list = Vec::new();
        let mut remaining = &encoded_value[1..];

        while !remaining.is_empty() && !remaining.starts_with('e') {
            let (decoded_value, rest) = decode_bencoded_value(remaining);
            list.push(decoded_value);
            remaining = rest;
        }

        (list.into(), &remaining[1..])
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
