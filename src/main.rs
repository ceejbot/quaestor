use colored::*;
use reqwest::Url;
use serde::Deserialize;
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
    #[structopt(help = "quaestor dump\n    emit all values in the database; use with care")]
    Dump {
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

#[derive(Debug)]
struct KeyPair<'a> {
    value: Option<&'a str>,
    child: HashMap<&'a str, KeyPair<'a>>
}

fn build_entry<'a>() -> KeyPair<'a> {
    KeyPair {
        value: None,
        child: HashMap::new()
    }
}

fn dir<'a>(prefix: &str) -> anyhow::Result<()> {
    let address = format!("http://localhost:8500/v1/kv/{}?recurse=true", prefix);
    let url = Url::parse(&address).unwrap();
    let values: Vec<ConsulValue> = reqwest::get(url)?.json()?;

    // let's do this the stupidest possible way.
    let mut result: KeyPair = build_entry();

    for v in &values {
        let bytes = base64::decode(&v.Value).unwrap();
        println!("{} = {}", v.Key.blue(), std::str::from_utf8(&bytes)?.green());

        let mut segments = v.Key.split("/").collect::<Vec<&str>>();
        segments.reverse();

        let mut current: &mut KeyPair = &mut result;

        while segments.len() > 1 {
            let level = segments.pop().unwrap();
            if !current.child.contains_key(level) {
                let tmp = build_entry();
                current.child.insert(level, tmp);
            }
            // and now my troubles begin
            let foo = current.child.get(level).unwrap();
            current = foo;
            // current = current.child.get(level).unwrap();
            // next:
            // make a new keypair
            // set its value to the last segment
            // insert it into the child of the current thingie
        }
    }

    println!("{:?}", result);

    Ok(())
}

fn dump() -> anyhow::Result<()> {
    Ok(())
}

fn main() {
    let opts = Commands::from_args();

    let res = match opts {
        Commands::Get { key }=> get(&key),
        Commands::Set { key, value  } => set(&key, &value),
        Commands::Dir { prefix } => dir(&prefix),
        Commands::Dump { } => dump(),
    };

    ::std::process::exit(match res {
        Err(e) => {
            eprintln!("error: {:?}", e);
            1
        },
        Ok(_) => 0,
    })
}
