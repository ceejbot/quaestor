use colored::*;
use reqwest::Url;
use std::fmt;
use structopt::StructOpt;

use serde::Deserialize;
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

// annoying error boilerplate
#[derive(Debug, Clone)]
pub enum QuaestorError {
    NotFound,
    ConsulError,
}

impl std::fmt::Display for QuaestorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuaestorError::NotFound => f.write_str("NotFound"),
            QuaestorError::ConsulError => f.write_str("ConsulError"),
        }
    }
}
impl std::error::Error for QuaestorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
    fn description(&self) -> &str {
        match *self {
            QuaestorError::NotFound => "Key not found",
            QuaestorError::ConsulError => "Consul failed surprisingly",
        }
    }
}

// end error boilerplate

fn get_json(target: &str) -> Result< ConsulValue, Box<dyn std::error::Error> > {
    let address = format!("http://localhost:8500/v1/kv/{}", target);
    let url = Url::parse(&address).unwrap();
    let values: Vec<ConsulValue> = reqwest::get(url)?.json()?;

    match values.len() {
        0 => Err(QuaestorError::NotFound.into()),
        _ => Ok(values[0].clone()),
    }
}

fn get(key: &str) -> Result<(), Box<dyn std::error::Error>> {
    match get_json(key) {
        Ok(v) => {
            // println!("{:?}", v);
            let bytes = base64::decode(&v.Value).unwrap();
            println!("{} = {}", key.blue(), std::str::from_utf8(&bytes)?.green());
        },
        Err(e) => { eprintln!("error: {:?}", e); },
    }
    Ok(())
}

fn set(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn dir(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("dir {}", prefix.blue());
    Ok(())
}

fn dump() -> Result<(), Box<dyn std::error::Error>> {
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
