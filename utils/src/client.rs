use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct ClientInfo {
    pub id: u128,
    pub account: String,
    pub passwd: Option<String>,
    pub ipv4s: Vec<Ipv4Addr>,
    pub ipv6s: Vec<Ipv6Addr>,
}

impl ClientInfo {
    pub fn new(id: u128, account: &str) -> Self {
        Self {
            id,
            account: account.to_owned(),
            passwd: None,
            ipv4s: vec![],
            ipv6s: vec![],
        }
    }

    pub fn builder(id: u128, account: &str, passwd_plaintext: &Option<String>) -> Self {
        let mut new_one = Self::new(id, account);
        /* Get the sha256 of the password if the password exists. */
        let passwd_sha256 = match passwd_plaintext {
            Some(plaintext) => {
                 let sha256ed = sha256(&plaintext);
                 Some(sha256ed)
            },
            None => None,
        };
        new_one.passwd = passwd_sha256;
        new_one
    }

    /// Set IPs by a vector of `IpAddr`, put an IPv4 in self `ipv4s`,
    /// and IPv6 in self `ipv6s`.
    pub fn set_ips(mut self, ips: &Vec<IpAddr>) -> Self {
        for ip in ips {
            match ip {
                IpAddr::V4(ipv4) => self.ipv4s.push(*ipv4),
                IpAddr::V6(ipv6) => self.ipv6s.push(*ipv6),
            }
        }
        self
    }

    /// Verify the hash between the password plaintext
    /// and the password hash of self, return true if same.
    pub fn verify_passwd(&self, passwd_plaintext: &str) -> bool {
        match &self.passwd {
            Some(p) => {
                let passwd_sha256ed = sha256(passwd_plaintext);
                if p == &passwd_sha256ed { true } else { false }
            },
            None => false,
        }
    }
}

/// A simple function for get an sha256ed hash from a plaintext.
fn sha256(plaintext: &str) -> String {
    use crypto::sha2::Sha256;
    use crypto::digest::Digest;
    let mut sha256er = Sha256::new();
    sha256er.input_str(plaintext);
    sha256er.result_str()    
}