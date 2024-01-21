use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_bencode;
use serde_json;
use sha1::{Digest, Sha1};

fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('0'..='9') => encoded_value
            .split_once(':')
            .and_then(|(len, remaining)| {
                Some(
                    len.parse::<usize>()
                        .and_then(|len| {
                            let (text, rest) = remaining.split_at(len);
                            Ok((serde_json::Value::String(text.to_string()), rest))
                        })
                        .expect("String case: Could not parse string length"),
                )
            })
            .expect("String case: Couldn't find delimiter ':'"),
        Some('i') => encoded_value
            .split_once('e')
            .and_then(|(unparsed_int, rest)| {
                Some((
                    serde_json::Value::Number(
                        unparsed_int[1..]
                            .parse::<isize>()
                            .expect("Int case: Could not parse int")
                            .into(),
                    ),
                    rest,
                ))
            })
            .expect("Int case: Couldn't find end 'e'"),
        Some('l') => {
            let mut list = Vec::new();
            let mut rest = &encoded_value[1..];

            while !rest.is_empty() && !rest.starts_with('e') {
                let (decoded_value, remaining) = decode_bencoded_value(rest);
                list.push(decoded_value);
                rest = remaining;
            }

            (list.into(), &rest[1..])
        }
        Some('d') => {
            let mut map = serde_json::Map::new();
            let mut rest = &encoded_value[1..];

            while !rest.is_empty() && !rest.starts_with('e') {
                let (key, rest_with_value) = decode_bencoded_value(rest); // key
                let k = match key {
                    serde_json::Value::String(s) => s,
                    _ => panic!("Key is not a string"),
                };
                let (value, remaining) = decode_bencoded_value(rest_with_value); // value
                map.insert(k, value);
                rest = remaining;
            }

            (serde_json::Value::Object(map), &rest[1..])
        }
        _ => panic!("Unhandled encoded value: {}", encoded_value),
    }
}

#[derive(Serialize, Deserialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo,
}

#[derive(Serialize, Deserialize)]
struct TorrentInfo {
    length: usize,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
}

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Decodes a given bencoded string
    Decode { bencode: String },
    /// Provides info about a given torrent file
    Info { file: String },
}

fn main() {
    match &Cli::parse().command {
        Some(Commands::Decode { bencode }) => {
            let decoded_value = decode_bencoded_value(bencode);
            println!("{}", decoded_value.0.to_string());
        }
        Some(Commands::Info { file }) => {
            let file_content = std::fs::read::<PathBuf>(file.into()).expect("Could not read file");
            let torrent = serde_bencode::from_bytes::<Torrent>(&file_content)
                .expect("Could not parse torrent file");
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);

            let encoded_info = serde_bencode::to_bytes(&torrent.info).expect("Could not encode");
            let mut hasher = Sha1::new();
            hasher.update(&encoded_info);
            let info_hash = hasher.finalize();

            println!("Info Hash: {}", hex::encode(&info_hash));

            println!("Piece Length: {}", torrent.info.piece_length);

            println!("Piece Hash:");
            torrent
                .info
                .pieces
                .chunks(20)
                .map(hex::encode)
                .for_each(|piece| {
                    println!("{}", piece);
                });
        }
        None => {}
    }
}
