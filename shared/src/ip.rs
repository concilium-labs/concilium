use concilium_error::Error;

pub fn ipv4_to_array(ip: &str) -> Result<[u8; 4], Error> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(Error::new(format!("Invalid IP address: {}", ip).as_str()));
    }

    let mut ip = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        match part.parse::<u8>() {
            Ok(num) => ip[i] = num,
            Err(_) => return Err(Error::new(format!("Invalid number in IP: {}", part).as_str())),
        }
    }

    Ok(ip)
}

pub fn ipv4_to_string(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

pub fn ipv6_to_array(ip: &str) -> Result<[u16; 8], Error> {
    let parts: Vec<&str> = ip.split(':').collect();
    if parts.len() != 8 {
        return Err(Error::new(format!("Invalid IPv6 address: {}", ip).as_str()));
    }

    let mut ip = [0u16; 8];
    for (i, part) in parts.iter().enumerate() {
        match u16::from_str_radix(part, 16) {
            Ok(num) => ip[i] = num,
            Err(_) => return Err(Error::new(format!("Invalid hex number in IPv6: {}", part).as_str())),
        }
    }

    Ok(ip)
}

pub fn ipv6_to_string(ip: &[u16; 8]) -> String {
    ip.iter()
        .map(|num| format!("{:x}", num))
        .collect::<Vec<String>>()
        .join(":")
}
