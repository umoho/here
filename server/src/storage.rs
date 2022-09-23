use std::{path::PathBuf, time::Duration};

use chrono::{DateTime, Utc, serde::ts_seconds};
use tinydb::{Database, error::DatabaseError};
use utils::client::ClientInfo;

use serde_derive::{Serialize, Deserialize};

use crate::restful::DATABASE_DUMPS_PATH;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug)]
pub(crate) struct ClientInfoRecord {
    pub(crate) client_info: ClientInfo,

    #[serde(with = "ts_seconds")]
    record_time: DateTime<Utc>,

    lifetime: u64,
}

impl ClientInfoRecord {
    /// Create a new record, and the record time is an UTC now.
    pub(crate) fn new(client_info: ClientInfo, lifetime: u64) -> Self {
        Self {
            client_info,
            record_time: DateTime::from(Utc::now()),
            lifetime,
        }
    }
}

/// Clean a outdated client information record.
pub(crate) fn clean_outdated(lifetime: u64) -> Result<(), DatabaseError> {
    /* Open (or create) a database. */
    let db = Database::auto_from(
        PathBuf::from(DATABASE_DUMPS_PATH),
        false
    )?;
    /* Query the item with `lifetime` param. Record with same lifetime will be select. */
    let item = db.query_item(
        |s: &ClientInfoRecord| {
            &s.lifetime
        },
        lifetime
    )?;
    /* Compare the item's lifetime with the param lifetime. */
    if is_outdated(item, lifetime) {
        #[cfg(feature = "debug-printing")] println!("Found an outdated storage: {:?}", item);
        /* Open (or create) the database. */
        let mut db = Database::auto_from(
            PathBuf::from(DATABASE_DUMPS_PATH),
            false
        )?;
        /* Remove a outdated item. */
        db.remove_item(item)?;
        /* This step is important. It makes changes write down to the file(s). */
        db.dump_db()?;
    }
    Ok(())
}

/// A simple function compare the lifetime of the record and `lifetime` param,
/// and return if the different between them is bigger then the `lifetime` param.
fn is_outdated(s: &ClientInfoRecord, lifetime: u64) -> bool {
    /* Get the different. */
    let diff = DateTime::from(Utc::now()) - s.record_time.to_owned();
    /* Cover the value `diff` to a `Duration` in `std`. */
    let diff_std = diff.to_std()
        .expect("Cannot cover the duration from `chrono` to `std` implementation.");
    /* Compare. Return `true` if bigger, or `false` otherwise. */
    diff_std > Duration::from_secs(lifetime)
}