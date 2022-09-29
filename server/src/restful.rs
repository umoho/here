use std::{net::SocketAddr, path::PathBuf};

use axum::{Router, routing::{get, post}, response::IntoResponse, http::StatusCode, Json, extract::Query};
use tinydb::{Database, error::DatabaseError};
use utils::{AppInfo, server::{GetClientInfoParams, PostClientInfoResponse, ResponseMessage, GetClientInfoResponse}, client::ClientInfo};

use crate::storage::ClientInfoRecord;

/// The API path to get server information.
const PATH_TO_GET_SERVER_INFO: &str = "/here/server";

/// The API path to get client information.
const PATH_TO_GET_CLIENT_INFO: &str = "/here/client/get";

/// The API path to post client information.
const PATH_TO_POST_CLIENT_INFO: &str = "/here/client/post";

/// The name of this App.
const APP_NAME: &str = "Here";

/// The version of this App.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The default lifetime of the client information record. Seconds.
#[cfg(feature = "debug-lifetime")]
pub(crate) const DEFAULT_LIFETIME: u64 = 5;

#[cfg(not(feature = "debug-lifetime"))]
pub(crate) const DEFAULT_LIFETIME: u64 = 60;

/// The path where the database file put.
pub(crate) const DATABASE_DUMPS_PATH: &str = "./client-info.db";

/// The summary (entry) function of the server.
pub(crate) async fn run_restful_api_server(addr: SocketAddr) -> Result<(), anyhow::Error> {
    /* Build an app by router. */
    let app = Router::new()
        .route(PATH_TO_GET_SERVER_INFO, get(get_server_info))
        .route(PATH_TO_GET_CLIENT_INFO, get(get_client_info))
        .route(PATH_TO_POST_CLIENT_INFO, post(post_client_info));

    /* Bind the address, and run the server. */
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// The get server information method.
async fn get_server_info() -> impl IntoResponse {
    /* Simply response an json of `AppInfo`. */
    (StatusCode::OK, Json(AppInfo::new(APP_NAME, APP_VERSION)))
}

/// The get client information method.
async fn get_client_info(Query(params): Query<GetClientInfoParams>) -> impl IntoResponse {
    /* Open (or create) the database. Response a server error when failed. */
    let db = match Database::auto_from(
            PathBuf::from(DATABASE_DUMPS_PATH), false
    ) {
        Ok(db) => db,
        Err(_) => {
            /* Build up a response with error message. */
            let resp = GetClientInfoResponse::new()
                .set_message(Some(ResponseMessage::DatabaseError));
            /* Response a `500` status code. */
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
        },
    };
    /* Query the item of record by account. */
    let item = match db.query_item(
        |r: &ClientInfoRecord| {
            &r.client_info.account
        },
        params.account
    ) {
        Ok(i) => i,
        Err(e) => {
            match e {
                DatabaseError::ItemNotFound => {
                    /* Build up a response with error message. */
                    let resp = GetClientInfoResponse::new()
                        .set_message(Some(ResponseMessage::NotFound));
                    /* Response a `404` status code. */
                    return (StatusCode::NOT_FOUND, Json(resp));
                },
                _ => {
                    /* Build up a response with error message. */
                    let resp = GetClientInfoResponse::new()
                        .set_message(Some(ResponseMessage::DatabaseError));
                    /* Response a `500` status code. */
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
                },
            }
        }
    };
    
    let client_info = &item.client_info;

    let passwd_plaintext =  match &params.passwd {
        Some(p) => p,
        None => {
            if client_info.passwd.is_some() {
                /* Build up a response with error message. */
                let resp = GetClientInfoResponse::new()
                    .set_message(Some(ResponseMessage::InvalidPassword));
                /* Response a `403` status code. */
                return (StatusCode::FORBIDDEN, Json(resp));
            }
            else {
                let resp = GetClientInfoResponse::new()
                    .set_ok(true);
                return (StatusCode::OK, Json(resp));
            }
        },
    };

    if !client_info.verify_passwd(passwd_plaintext) {
        /* Build up a response with error message. */
        let resp = GetClientInfoResponse::new()
            .set_message(Some(ResponseMessage::InvalidPassword));
        /* Response a `403` status code. */
        (StatusCode::FORBIDDEN, Json(resp))
    }
    else {
        let resp = GetClientInfoResponse::new()
            .set_ok(true).set_data(client_info);
        (StatusCode::OK, Json(resp))
    }
}

/// The post client information method.
async fn post_client_info(Json(client_info): Json<ClientInfo>) -> impl IntoResponse {
    #[cfg(feature = "debug-printing")] println!("A new post request from client, id = {}.", client_info.id);

    let client_lifetime = DEFAULT_LIFETIME;
    /* Open (or create) the database. Response a server error when failed. */
    let mut db = match Database::auto_from(
            PathBuf::from(DATABASE_DUMPS_PATH), false
    ) {
        Ok(db) => db,
        Err(_) => {
            /* Build up a response with error message. */
            let resp = PostClientInfoResponse::new(
                client_info.id, &client_info.account, client_info.passwd
            ).set_message(Some(ResponseMessage::DatabaseError));
            /* Response a `500` status code. */
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
        },
    };
    /* Add the record to the database. Response a server error when failed. */
    if let Err(_) = db.add_item(ClientInfoRecord::new(client_info.clone(), client_lifetime)) {
        /* Build up a response with error message. */
        let resp = PostClientInfoResponse::new(
            client_info.id, &client_info.account, client_info.passwd
        ).set_message(Some(ResponseMessage::DatabaseError));
        /* Response a `500` status code. */
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
    }
    /* Dump the data to the database, and then the data will be storage. */
    if let Err(_) = db.dump_db() {
        /* Build up a response with error message. */
        let resp = PostClientInfoResponse::new(
            client_info.id, &client_info.account, client_info.passwd
        ).set_message(Some(ResponseMessage::DatabaseError));
        /* Response a `500` status code. */
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
    }

    /* If success after those steps, send a ok response with a lifetime. */
    let resp = PostClientInfoResponse::new(
            client_info.id, &client_info.account, client_info.passwd
        )
        .set_ok(true)
        .set_lifetime(client_lifetime);
    /* Response a `200` status code. */
    (StatusCode::OK, Json(resp))
}