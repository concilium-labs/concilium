use concilium_core::node::SelfNode;

pub trait SelfNodeSupport {
    fn new(id: u32, name: Vec<u8>, public_key: [u8; 48], private_key: [u8; 32], ip_address: [u8; 4], port: u16, version: Vec<u8>, created_at: i64) -> Self;
    fn get_id(&self) -> u32;
    fn get_name(&self) -> &[u8];
    fn get_public_key(&self) -> &[u8; 48];
    fn get_private_key(&self) -> &[u8; 32];
    fn get_ip_address(&self) -> &[u8; 4];
    fn get_port(&self) -> u16;
    fn get_version(&self) -> &[u8];
    fn get_created_at(&self) -> i64;
    fn get_self(&self) -> &Self;
    fn set_id(&mut self, id: u32);
    fn set_name(&mut self, name: Vec<u8>);
    fn set_public_key(&mut self, public_key: [u8; 48]);
    fn set_private_key(&mut self, private_key: [u8; 32]);
    fn set_ip_address(&mut self, ip_address: [u8; 4]);
    fn set_port(&mut self, port: u16);
    fn set_version(&mut self, version: Vec<u8>);
    fn set_created_at(&mut self, created_at: i64);
}

impl SelfNodeSupport for SelfNode {
    fn new(id: u32, name: Vec<u8>, public_key: [u8; 48], private_key: [u8; 32], ip_address: [u8; 4], port: u16, version: Vec<u8>, created_at: i64) -> Self {
        Self {
            id,
            name,
            public_key,
            private_key,
            ip_address,
            port,
            version,
            created_at
        }
    }

    fn get_id(&self) -> u32 {
        self.id
    }
    
    fn get_name(&self) -> &[u8] {
        &self.name
    }
    
    fn get_public_key(&self) -> &[u8; 48] {
        &self.public_key
    }
    
    fn get_private_key(&self) -> &[u8; 32] {
        &self.private_key
    }

    fn get_ip_address(&self) -> &[u8; 4] {
        &self.ip_address
    }
    
    fn get_port(&self) -> u16 {
        self.port
    }

    fn get_version(&self) -> &[u8] {
        &self.version
    }

    fn get_created_at(&self) -> i64 {
        self.created_at
    }
    
    fn get_self(&self) -> &Self{
        &self
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn set_name(&mut self, name: Vec<u8>) {
        self.name = name;
    }
    
    fn set_public_key(&mut self, public_key: [u8; 48]) {
        self.public_key = public_key;
    }
    
    fn set_private_key(&mut self, private_key: [u8; 32]) {
        self.private_key = private_key;
    }

    fn set_ip_address(&mut self, ip_address: [u8; 4]) {
        self.ip_address = ip_address;
    }
    
    fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    fn set_version(&mut self, version: Vec<u8>) {
        self.version = version;
    }

    fn set_created_at(&mut self, created_at: i64) {
        self.created_at = created_at;
    }
}
