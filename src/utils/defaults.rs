pub mod http {
    use std::net::{Ipv4Addr, SocketAddrV4};

    pub fn address() -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)
    }
}

pub mod database {
    pub fn file() -> String {
        "mordor.db".to_string()
    }
}

pub const fn store_access_entries() -> bool {
    false
}
