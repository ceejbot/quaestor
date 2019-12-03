use colored::*;
use reqwest::Url;
use serde::Deserialize;
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

fn dir(prefix: &str) -> anyhow::Result<()> {
    println!("dir {}", prefix.blue());
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
            1 },
        Ok(_) => 0,
    })
}
