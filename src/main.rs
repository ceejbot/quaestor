use colored::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{ BufRead, BufReader };
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
    let result = reqwest::get(&address)?.text();

    match result {
        Ok(v) => {
            if v.len() > 0 {
                println!("{} = {}", key.blue(), v.green());
            } else {
                println!("‼️  {} not found!", key.red());
            }
        },
        Err(e) => { eprintln!("‼️  error: {:?}", e); },
    }
    Ok(())
}

fn set(key: &str, value: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}", key);
    let response = reqwest::Client::new()
        .put(&address)
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
    let response = reqwest::Client::new()
        .delete(&address)
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
    let values: Vec<ConsulValue> = reqwest::get(&address)?.json()?;

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

#[derive(Deserialize, Serialize, Debug)]
struct Entry(HashMap<String, String>);

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

fn import<R: BufRead>(mut reader: R, fname: String) -> anyhow::Result<()> {
    let mut count_new = 0;
    let mut count_replace = 0;

    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    let content = std::str::from_utf8(&data)?;
    let imports: Entry = serde_json::from_str(content)?;

    for (key, value) in imports.0 {
        let address = format!("http://localhost:8500/v1/kv/{}", key);
        let mut get_resp = reqwest::get(&address)?;

        let modify_index = if get_resp.status().as_u16() == 404 {
            0
        } else {
            let mut values: Vec<ConsulValue> = get_resp.json()?;
            values.pop().unwrap().ModifyIndex
        };

        let address = format!("http://localhost:8500/v1/kv/{}?cas={}", key, modify_index);
        let _set_resp = reqwest::Client::new()
            .put(&address)
            .body(value)
            .send()?;
        if modify_index == 0 {
            count_new += 1;
        } else {
            count_replace += 1;
        }
    }

    println!("Finished import from {}.", fname.bold());
    if count_new == 1 {
        println!("Added 1 new value.");
    } else if count_new > 1 {
        println!("Added {} new values.", count_new);
    }
    if count_replace == 1 {
        println!("Updated 1 value.");
    } else if count_replace > 1 {
        println!("Updated {} values.", count_replace);
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts = Commands::from_args();

    let res = match opts {
        Commands::Get { key }=> get(&key),
        Commands::Set { key, value  } => set(&key, &value),
        Commands::Rm { key } => remove(&key),
        Commands::Dir { prefix } => dir(&prefix),
        Commands::Export { } => export(),
        Commands::Import { fpath } => {

            if fpath == "-" {
                import(BufReader::new(io::stdin()), "<stdin>".to_string())
            } else {
                let file = File::open(&fpath)?;
                let reader = BufReader::new(file);
                import(reader, fpath)
            }
        },
    };

    ::std::process::exit(match res {
        Err(e) => {
            eprintln!("‼️  fatal error: {:?}", e);
            1
        },
        Ok(_) => 0,
    })
}
