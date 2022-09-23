use serde_derive::{Serialize, Deserialize};

use crate::client::ClientInfo;

/// The param form of get client info requests.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetClientInfoParams {
    pub account: String,
    pub passwd: Option<String>,
}

/// The response form of get client info requests.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetClientInfoResponse {
    id: Option<u128>,
    account: String,
    passwd: Option<String>,
    is_ok: bool,
    message: Option<ResponseMessage>,
    data: Option<ClientInfo>,
}

impl GetClientInfoResponse {
    pub fn new(id: Option<u128>, account: &str, passwd: Option<String>) -> Self {
        Self {
            id,
            account: account.to_owned(),
            passwd,
            is_ok: false,
            message: None,
            data: None,
        }
    }

    pub fn set_ok(mut self, is_ok: bool) -> Self {
        self.is_ok = is_ok;
        self
    }

    pub fn set_message(mut self, message: Option<ResponseMessage>) -> Self {
        self.message = message;
        self
    }

    pub fn set_id(mut self, id: Option<u128>) -> Self {
        self.id = id;
        self
    }

    pub fn set_data(mut self, data: ClientInfo) -> Self {
        self.data = Some(data);
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostClientInfoResponse {
    id: u128,
    account: String,
    passwd: Option<String>,
    is_ok: bool,
    message: Option<ResponseMessage>,
    lifetime: u64,
}

impl PostClientInfoResponse {
    pub fn new(id: u128, account: &str, passwd: Option<String>) -> Self {
        Self {
            id,
            account: account.to_owned(),
            passwd,
            is_ok: false,
            message: None,
            lifetime: 0,
        }
    }

    pub fn set_ok(mut self, is_ok: bool) -> Self {
        self.is_ok = is_ok;
        self
    }

    pub fn set_message(mut self, message: Option<ResponseMessage>) -> Self {
        self.message = message;
        self
    }

    pub fn set_lifetime(mut self, lifetime: u64) -> Self {
        self.lifetime = lifetime;
        self
    }

    pub fn lifetime(&self) -> u64 {
        self.lifetime
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseMessage {
    NotFound,
    AlreadyOccupiedId,
    InvalidPassword,
    DatabaseError,
}