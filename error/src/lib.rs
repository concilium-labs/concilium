#[derive(Debug)]
pub struct Error {
    message: String
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            message: format!("IO Error: {}", error.to_string())
        }
    }
}

impl From<rocksdb::Error> for Error {
    fn from(error: rocksdb::Error) -> Self {
        Self {
            message: format!("RocksDB Error: {}", error.to_string())
        }
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(error: bincode::error::EncodeError) -> Self {
        Self {
            message: format!("Bincode(Encode) Error: {}", error.to_string())
        }
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(error: bincode::error::DecodeError) -> Self {
        Self {
            message: format!("Bincode(Decode) Error: {}", error.to_string())
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self {
            message: format!("String Error: {}", error.to_string())
        }
    }
}

impl From<std::string::ParseError> for Error {
    fn from(error: std::string::ParseError) -> Self {
        Self {
            message: format!("Parse Error: {}", error.to_string())
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self {
            message: format!("ParseInt Error: {}", error.to_string())
        }
    }
}

impl From<std::str::ParseBoolError> for Error {
    fn from(error: std::str::ParseBoolError) -> Self {
        Self {
            message: format!("ParseBool Error: {}", error.to_string())
        }
    }
}

impl From<std::char::ParseCharError> for Error {
    fn from(error: std::char::ParseCharError) -> Self {
        Self {
            message: format!("ParseChar Error: {}", error.to_string())
        }
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(error: std::num::ParseFloatError) -> Self {
        Self {
            message: format!("ParseFloat Error: {}", error.to_string())
        }
    }
}

impl From<std::env::VarError> for Error {
    fn from(error: std::env::VarError) -> Self {
        Self {
            message: format!("Var Error: {}", error.to_string())
        }
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(error: tonic::transport::Error) -> Self {
        Self {
            message: format!("Tonic(Transport) Error: {}", error.to_string())
        }
    }
}

impl From<tonic::Status> for Error {
    fn from(error: tonic::Status) -> Self {
        Self {
            message: format!("Tonic(Status) Error: {}", error.to_string())
        }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(error: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self {
            message: format!("Tokio(Send) Error: {}", error.to_string())
        }
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(error: std::net::AddrParseError) -> Self {
        Self {
            message: format!("Net(AddrParseError) Error: {}", error.to_string())
        }
    }
}

impl From<dotenvy::Error> for Error {
    fn from(error: dotenvy::Error) -> Self {
        Self {
            message: format!("Dotenvy Error: {}", error.to_string())
        }
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(error: std::num::TryFromIntError) -> Self {
        Self {
            message: format!("TryFromIntError: {}", error.to_string())
        }
    }
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(error: std::array::TryFromSliceError) -> Self {
        Self {
            message: format!("TryFromSliceError: {}", error.to_string())
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(error: hex::FromHexError) -> Self {
        Self {
            message: format!("FromHexError: {}", error.to_string())
        }
    }
}

impl From<Vec<u8>> for Error {
    fn from(_: Vec<u8>) -> Self {
        Self {
            message: "Vec<u8>".to_string()
        }
    }
}

impl From<jsonrpsee::core::ClientError> for Error {
    fn from(error: jsonrpsee::core::ClientError) -> Self {
        Self {
            message: format!("JsonrpseeClientError: {}", error.to_string())
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self {
            message: format!("SerdeJsonError: {}", error.to_string())
        }
    }
}

impl From<blst::BLST_ERROR> for Error {
    fn from(error: blst::BLST_ERROR) -> Self {
        let error_msg = match error {
            blst::BLST_ERROR::BLST_SUCCESS => String::from("BLST_SUCCESS"),
            blst::BLST_ERROR::BLST_BAD_ENCODING => String::from("BLST_BAD_ENCODING"),
            blst::BLST_ERROR::BLST_POINT_NOT_ON_CURVE => String::from("BLST_POINT_NOT_ON_CURVE"),
            blst::BLST_ERROR::BLST_POINT_NOT_IN_GROUP => String::from("BLST_POINT_NOT_IN_GROUP"),
            blst::BLST_ERROR::BLST_AGGR_TYPE_MISMATCH => String::from("BLST_AGGR_TYPE_MISMATCH"),
            blst::BLST_ERROR::BLST_VERIFY_FAIL => String::from("BLST_VERIFY_FAIL"),
            blst::BLST_ERROR::BLST_PK_IS_INFINITY => String::from("BLST_PK_IS_INFINITY"),
            blst::BLST_ERROR::BLST_BAD_SCALAR => String::from("BLST_BAD_SCALAR"),
        };
        Self {
            message: format!("Blst Error: {}", error_msg)
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.message.fmt(formatter)
    }
}