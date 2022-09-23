use std::net::IpAddr;

pub(crate) fn my_ip() -> Result<IpAddr, anyhow::Error> {
    use local_ip_address::local_ip;
    Ok(local_ip()?)
}

#[test]
fn test_my_ip() {
    println!("{:?}", my_ip());
}