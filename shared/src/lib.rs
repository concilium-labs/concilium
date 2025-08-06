pub mod binary;
pub mod coventor;
pub mod chacha20;
pub mod epoch;
pub mod ip;
pub mod sha;
pub mod transaction;

/*
    index 0 => public key
    index 1 => node address
*/
pub const BOOTSTRAP_NODES: [[&str; 2]; 5] = [
    ["8f99cef042f91a184f2d883448f4fc16a9b6e15d0de133c3749c5202a2a336abc575243f55afd9a48200e822036f8885", "127.0.0.1:50000"],
    ["b91bf3914f130751bb353181d3788558e61ddaa2f35a6b994263d9c8438b4fc676b452cffb2076c5e4d3c7d65ed50bcb", "127.0.0.1:50001"],
    ["8098d90ad324086d4565355278485648d1697b3b48b47084a774fce17a44dbd8ff27c9c015010796ef0a126b6ce4b7f2", "127.0.0.1:50002"],
    ["abec323085be1f9a772b2b85838ec830a8c5f00db820eebc4f79f8e2ab1377a41d36d364f0b747623cb4ec5434ea6079", "127.0.0.1:50003"],
    ["950d43b8239735ddffe27e9f38465ed1b2cfc305255ab8495a08536283c7ed1ce0a3a5b41f4f4fea2e82f1a228f7ebed", "127.0.0.1:50004"],
];

pub const DST: &[u8; 43] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";