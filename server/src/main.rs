use std::net::SocketAddr;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use storage::ClientInfoRecord;
use tinydb::Database;
use tinydb::error::DatabaseError;
use serde_derive::{Serialize, Deserialize};

use crate::restful::{DEFAULT_LIFETIME, DATABASE_DUMPS_PATH};
use crate::storage::clean_outdated;

/// About the RESTful API server.
mod restful;

/// About the database and storages.
mod storage;

/// Cleaning frequent, usually very short. Seconds.
const CLEAN_FREQUENT: f64 = 0.5;

/// Delay when finished to clean. Seconds.
const FINISHED_CLEAR_RELAX_DELAY: f64 = 10.0;

/// Delay when error to clean. Seconds.
const ERROR_TO_CLEAN_DELAY: f64 = 10.0;

/// Default config file put at this path.
const DEFAULT_CONFIG_PATH: &str = "./server.conf.toml";

#[derive(Serialize, Deserialize)]
struct Config {
    bind: String,
}

#[tokio::main]
async fn main() {
    /* Set the Ctrl-C handler. */
    ctrlc::set_handler(|| {
        eprintln!("Server stop.");
        std::process::exit(0);
    }).expect("Cannot set Ctrl-C handler.");

    /* Load config from the file. */
    println!("Loading config...");
    let config = get_config(DEFAULT_CONFIG_PATH).expect("Cannot load config.");

    /* Whatever, test if the database exists. It will be create if not exist. */
    Database::<ClientInfoRecord>::auto_from(PathBuf::from(DATABASE_DUMPS_PATH), false).expect("Database error.");

    /* The thread of cleaning the outdated storages of client information. */
    let cleaning = cleaning_thread();

    /* Start the RESTful API server. Listening on the binding address load from the config. */
    let bind_addr: SocketAddr = config.bind.parse().expect("Cannot parse the bind address. Please check the config.");
    println!("Starting the RESTful API server...\nListening on {}...", bind_addr);
    restful::run_restful_api_server(SocketAddr::from(bind_addr))
        .await.expect("Cannot run the RESTful server.");

    /* Let this function join main thread. */
    cleaning.join().expect("Thread cleaning joining error.");
}

/// Return a `JoinHandle<()>` struct, the spawned thread.
fn cleaning_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        loop {
            match clean_outdated(DEFAULT_LIFETIME) {
                Ok(_) => {
                    #[cfg(feature = "debug-printing")] println!("Successfully cleaned outdated storage of client information.");
                    /* Have a (very short time) relax. */
                    thread::sleep(Duration::from_secs_f64(CLEAN_FREQUENT));
                },
                Err(e) => {
                    match e {
                        DatabaseError::ItemNotFound => {
                            #[cfg(feature = "debug-printing")] println!("Finished to clean the outdated storages in the database.");
                            /* If we can'n find the item, it's possibly be cleaned. Have a relax. */
                            thread::sleep(Duration::from_secs_f64(FINISHED_CLEAR_RELAX_DELAY));
                        }
                        _ => {
                            eprintln!("Failed to clean the outdated storages.");
                            thread::sleep(Duration::from_secs_f64(ERROR_TO_CLEAN_DELAY));
                        },
                    }
                },
            }
        }
    })
}

/// Read the config file at `path`, or create a new one
/// by default config if the file not exists.
fn get_config(path: &str) -> Result<Config, anyhow::Error> {
    use std::fs::File;
    use std::io::Write;

    use std::io::{stdin, stdout, Read};
    /* Try to open the config file. Create a new one if it is not exists. */
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            /* Read config from the cmdline. */
            print!("Please input the bind (example: 127.0.0.1:8080): ");
            stdout().flush()?;
            let mut bind = String::new();
            stdin().read_line(&mut bind)?;
            let bind = bind.trim().to_owned();

            /* Build a new config by default. */
            let default_config = Config {
                bind
            };

            /* Create a file and write contents. */
            let mut new_file = File::create(path)?;
            let buf = toml::to_vec(&default_config)?;
            new_file.write(&buf)?;

            /* Return the default config. */
            return Ok(default_config);
        },
    };
    /* Read the config from the file. */
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = toml::from_str(&contents)?;
    Ok(config)
}