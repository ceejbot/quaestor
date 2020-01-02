use colored::*;
use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
struct ConsulValue {
    CreateIndex: u32,
    Flags: u32,
    Key: String,
    LockIndex: u32,
    ModifyIndex: u32,
    Value: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "quaestor")]
enum Commands {
    #[structopt(help = "quaestor set key value\n    set a key")]
    Set {
        key: String,
        value: String,
    },
    #[structopt(help = "quaestor get key\n    get a key")]
    Get {
        key: String,
    },
    #[structopt(help = "quaestor dir prefix\n    recursively get a key and all values that start with that prefix")]
    Dir {
        prefix: String,
    },
    #[structopt(help = "quaestor dump\n    emit all values in the database to json; use with care")]
    Dump {
    },
    #[structopt(help = "quaestor import filepath\n    import key/value pairs from a JSON file")]
    Import {
        fpath: String
    },
}

fn get(key: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}?raw=true", key);
    let url = Url::parse(&address).unwrap();
    let result = reqwest::get(url)?.text();

    match result {
        Ok(v) => {
            if v.len() > 0 {
                println!("{} = {}", key.blue(), v.green());
            } else {
                println!("{} not found!", key.red());
            }
        },
        Err(e) => { eprintln!("error: {:?}", e); },
    }
    Ok(())
}

fn set(key: &str, value: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}", key);
    let url = Url::parse(&address).unwrap();

    let response = reqwest::Client::new()
        .put(url)
        .body(String::from(value))
        .send()?;

    match response.error_for_status() {
        Ok(_res) => {
            println!("{} ➜ {}", key.blue(), value.green());
        },
        Err(e) => {
            println!("failed to set {}! {:?}", key.blue(), e);
        },
    }

    Ok(())
}

#[derive(Serialize, Debug)]
enum KeyPair {
    String(String),
    Object(HashMap<String, KeyPair>)
}

fn dir<'a>(prefix: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}?recurse=true", prefix);
    let url = Url::parse(&address).unwrap();
    let values: Vec<ConsulValue> = reqwest::get(url)?.json()?;

    // let's do this the stupidest possible way.
    let mut result: HashMap<String, KeyPair> = HashMap::new();

    for v in &values {
        let bytes = match base64::decode(&v.Value) {
            Err(_) => continue,
            Ok(b) => b,
        };

        let decoded = std::str::from_utf8(&bytes)?.to_string();

        let mut segments = v.Key.split("/").collect::<Vec<&str>>();
        segments.reverse();

        let mut current = &mut result;

        while segments.len() > 1 {
            let level = segments.pop().unwrap();
            let tmp = current.entry(level.to_string()).or_insert(KeyPair::Object(HashMap::new()));
            current = match tmp {
                KeyPair::String(_s) => panic!("Got string node at {} but it was not a terminal node", level),
                KeyPair::Object(v) => v
            };
        }
        let terminal = KeyPair::String(decoded);
        current.insert(segments.pop().unwrap().to_string(), terminal);
    }

    emit_level(&result, 0);
    Ok(())
}

fn emit_level(item: &HashMap<String, KeyPair>, level: usize) {
    for (k, v) in item.iter() {
        match v {
            KeyPair::String(val) => println!("{:width$}{}: {}", "", k.blue(), val.green(), width = level * 4),
            KeyPair::Object(next) => {
                println!("{:width$}{}:", "", k, width = level * 4);
                emit_level(next, level + 1);
            },
        }
    }
}

fn dump() -> anyhow::Result<()> {
    let values: Vec<ConsulValue> = reqwest::get("http://localhost:8500/v1/kv/?recurse=true")?.json()?;

    let mut result: HashMap<String, String> = HashMap::new();
    for v in &values {
        let bytes = match base64::decode(&v.Value) {
            Err(_) => continue,
            Ok(b) => b,
        };

        let decoded = std::str::from_utf8(&bytes)?.to_string();
        result.insert(v.Key.to_owned(), decoded);
    }

    let json = serde_json::to_string_pretty(&result)?;
    println!("{}", json);

    Ok(())
}

fn import(_fpath: &str) -> anyhow::Result<()> {
    Ok(())
}

fn main() {
    let opts = Commands::from_args();

    let res = match opts {
        Commands::Get { key }=> get(&key),
        Commands::Set { key, value  } => set(&key, &value),
        Commands::Dir { prefix } => dir(&prefix),
        Commands::Dump { } => dump(),
        Commands::Import { fpath } => import(&fpath),
    };

    ::std::process::exit(match res {
        Err(e) => {
            eprintln!("error: {:?}", e);
            1
        },
        Ok(_) => 0,
    })
}
