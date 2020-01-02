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
    #[structopt(help = "quaestor rm key\n    remove a key")]
    Rm {
        key: String
    },
    #[structopt(help = "quaestor dir prefix\n    recursively get a key and all values that start with that prefix")]
    Dir {
        prefix: String,
    },
    #[structopt(help = "quaestor export\n    emit all values in the database to json; use with care")]
    Export {
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
                println!("‼️  {} not found!", key.red());
            }
        },
        Err(e) => { eprintln!("‼️ error: {:?}", e); },
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
            println!("✔️ {} ➜ {}", key.blue(), value.green());
        },
        Err(e) => {
            println!("‼️ failed to set {}! {:?}", key.blue(), e);
        },
    }

    Ok(())
}

fn remove(key: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}", key);
    let url = Url::parse(&address).unwrap();

    let response = reqwest::Client::new()
        .delete(url)
        .send()?;

    match response.error_for_status() {
        Ok(_res) => {
            println!("✘ {} removed", key.red());
        },
        Err(e) => {
            println!("‼️ failed to remove {}! {:?}", key.blue(), e);
        },
    }

    Ok(())
}

#[derive(Serialize, Debug, Default)]
struct KeyPair {
    value: Option<String>,
    child: HashMap<String, KeyPair>
}

fn dir<'a>(prefix: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}?recurse=true", prefix);
    let url = Url::parse(&address).unwrap();
    let values: Vec<ConsulValue> = reqwest::get(url)?.json()?;

    // let's do this the stupidest possible way.
    let mut result = KeyPair::default();

    for v in &values {
        let bytes = match base64::decode(&v.Value) {
            Err(_) => continue,
            Ok(b) => b,
        };

        let decoded = std::str::from_utf8(&bytes)?.to_string();

        let mut segments: Vec<_> = v.Key.split("/").map(str::to_string).collect();
        segments.reverse();

        let mut current = &mut result;

        while segments.len() > 1 {
            let level = segments.pop().unwrap();
            let tmp = current.child.entry(level).or_insert(KeyPair::default());
            current = tmp;
        }
        let terminal = current.child.entry(segments.pop().unwrap()).or_insert(KeyPair::default());
        terminal.value = Some(decoded);
    }

    emit_level(result, -1, "".to_string());
    Ok(())
}

fn emit_level(item: KeyPair, level: i8, key: String) {
    if let Some(val) = item.value {
        println!("{:width$}{}: {}", "", key.blue(), val.green(), width = level as usize * 4);
    } else if key.len() > 0 {
        println!("{:width$}{}:", "", key, width = level as usize * 4);
    }

    for (k, v) in item.child.into_iter() {
        emit_level(v, level + 1, k);
    }
}

fn export() -> anyhow::Result<()> {
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
        Commands::Rm { key } => remove(&key),
        Commands::Dir { prefix } => dir(&prefix),
        Commands::Export { } => export(),
        Commands::Import { fpath } => import(&fpath),
    };

    ::std::process::exit(match res {
        Err(e) => {
            eprintln!("‼️ error: {:?}", e);
            1
        },
        Ok(_) => 0,
    })
}
