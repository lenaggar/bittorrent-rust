use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    if encoded_value.starts_with(|c: char| c.is_digit(10)) {
        return match encoded_value.split_once(':') {
            None => panic!("Could not split string, no ':' found"),
            Some((text_len, remaining)) => {
                let number = text_len.parse::<usize>().unwrap();
                (
                    serde_json::Value::String(remaining[..number].to_string()),
                    &remaining[number..],
                )
            }
        };
    } else if encoded_value.starts_with('i') {
        return match encoded_value.split_once('e') {
            None => panic!("Could not split string, no 'e' found"),
            Some((number, remaining)) => (
                serde_json::Value::Number(number[1..].parse::<isize>().unwrap().into()),
                remaining,
            ),
        };
    } else if encoded_value.starts_with('l') {
        let mut list = Vec::new();
        let mut remaining = &encoded_value[1..];

        while !remaining.is_empty() && !remaining.starts_with('e') {
            let (decoded_value, rest) = decode_bencoded_value(remaining);
            list.push(decoded_value);
            remaining = rest;
        }

        (list.into(), &remaining[1..])
    } else if encoded_value.starts_with('d') {
        let mut map = serde_json::Map::new();

        let mut remaining = &encoded_value[1..];

        while !remaining.is_empty() && !remaining.starts_with('e') {
            let (key, rest_with_value) = decode_bencoded_value(remaining); // key
            let k = match key {
                serde_json::Value::String(s) => s,
                _ => panic!("Key is not a string"),
            };
            let (value, rest) = decode_bencoded_value(rest_with_value); // value
            map.insert(k, value);
            remaining = rest;
        }

        (serde_json::Value::Object(map), &remaining[1..])
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
