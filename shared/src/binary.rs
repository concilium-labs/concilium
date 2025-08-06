use bincode::config::{
    self, 
    Configuration,
};
use concilium_error::Error;
use serde::{de::DeserializeOwned, ser::Serialize};

const BINCODE_CONFIG: Configuration = config::standard();

pub fn encode<T>(src: &T) -> Result<Vec<u8>, Error> 
where 
T: Serialize
{
    Ok(bincode::serde::encode_to_vec(src, BINCODE_CONFIG)?)
}

pub fn decode<T>(src: &[u8]) -> Result<T, Error> 
where 
T: DeserializeOwned
{
    let (data, _): (T, _) = bincode::serde::decode_from_slice(src, BINCODE_CONFIG)?;
    Ok(data)
}