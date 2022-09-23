use std::io::Read;
use std::{thread, time};

use serde_derive::{Deserialize, Serialize};
use reqwest::header::HeaderMap;

use utils::{client::ClientInfo, server::PostClientInfoResponse, AppInfo};

mod info;

/// The seconds of sleeping.
const SLEEP_SECONDS: f64 = 1.0;

/// Default config file put at this path.
const DEFAULT_CONFIG_PATH: &str = "./client.conf.toml";

#[derive(Serialize, Deserialize)]
struct Config {
    account: String,
    passwd: Option<String>,
    api_url: String,
}

#[tokio::main]
async fn main() {
    /* Set the Ctrl-C handler. */
    ctrlc::set_handler(|| {
        eprintln!("Quit.");
        std::process::exit(0);
    }).expect("Cannot set Ctrl-C handler.");

    /* Load config from the file. */
    println!("Loading config...");
    let config = get_config(DEFAULT_CONFIG_PATH).expect("Cannot load config.");

    /* Test network linking, and the server app version. */
    let server_info = loop {
        match get_server_info(&format!("{}/server", config.api_url)).await {
            Ok(info) => {
                println!("Listening to server response of app information...");
                break info;
            },
            Err(_) => {
                /* Sleep a second. */
                eprintln!("Cannot get the app information from the server yet.\nSleeping... Retry after {} second(s).", SLEEP_SECONDS);
                thread::sleep(time::Duration::from_secs_f64(SLEEP_SECONDS));
                /* Continue to post. */
                continue;
            },
        }
    };
    println!("Got the app information: {}", server_info);

    loop {
        /* Read my IPs. */
        let my_ips = vec![info::my_ip().expect("Cannot read my IP.")];
        /* Build my information. */
        let my_info = ClientInfo::builder(rand::random(), &config.account, &config.passwd).set_ips(&my_ips);
        /* Post my information. */
        match post_my_info(&format!("{}/client/post", config.api_url), &my_info).await {
            Ok(resp) => {
                /* We success to post our information. */
                let lifetime = resp.lifetime();
                #[cfg(feature = "debug-printing")] println!("Server response: {:?}", resp);
                /* Nothing to do. */
                println!("Successfully posted.\nSleeping... Redo post after {} second(s).", lifetime);
                thread::sleep(time::Duration::from_secs(lifetime));
                /* Redo after run out the lifetime. */
                continue;
            },
            Err(_) => {
                /* Sleep a second. */
                eprintln!("Cannot post my information.\nSleeping... Retry after {} second(s).", SLEEP_SECONDS);
                thread::sleep(time::Duration::from_secs_f64(SLEEP_SECONDS));
                /* Continue to post. */
                continue;
            },
        }
    }
}

/// Send to server a get request, and take back an `AppInfo` response.
async fn get_server_info(server_url: &str) -> Result<AppInfo, anyhow::Error> {
    /* Get server response. */
    let resp = reqwest::get(server_url).await?;
    /* Parse the server response into an `AppInfo` struct. */
    Ok(resp.json().await?)
}

/// Send to server a post request, and take back an `PostClientInfoResponse` response.
async fn post_my_info(server_url: &str, info: &ClientInfo) -> Result<PostClientInfoResponse, anyhow::Error> {
    let client = reqwest::Client::new();

    /* Build up a header. */
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    /* Post my information, then get response from the server. */
    let resp = client.post(server_url).headers(headers).json(&info).send().await?;

    /* Parse the server response into a `PostClientInfoResponse` struct. */
    Ok(resp.json().await?)
}

/// Read the config file at `path`, or create a new one
/// by default config if the file not exists.
fn get_config(path: &str) -> Result<Config, anyhow::Error> {
    use std::fs::File;
    use std::io::Write;

    use std::io::{stdin, stdout};
    /* Try to open the config file. Create a new one if it is not exists. */
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            /* Read config from the cmdline. */
            print!("Please input the account: ");
            stdout().flush()?;
            let mut account = String::new();
            stdin().read_line(&mut account)?;
            let account = account.trim().to_owned();

            print!("Please input the password (leave it empty for no password): ");
            stdout().flush()?;
            let mut passwd = String::new();
            stdin().read_line(&mut passwd)?;
            /* Set no password if leave it blank. */
            let passwd = if passwd.trim().is_empty() { Some(passwd) } else { None };

            print!("Please input the API URL (example: http://localhost/here): ");
            stdout().flush()?;
            let mut api_url = String::new();
            stdin().read_line(&mut api_url)?;
            let api_url = api_url.trim().to_owned();

            /* Build a new config by default. */
            let default_config = Config {
                account,
                passwd,
                api_url,
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