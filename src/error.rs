#![allow(dead_code)]

#[derive(Debug, Clone, thiserror::Error)]
pub enum RingError {
    #[error("invaild config")]
    InvalidConfig(String),

    #[error("internal error")]
    InternalError,

    #[error("invalid buffer size")]
    InvalidBufferSize,

    #[error("invalid packet")]
    InvalidPacket,

    #[error("timeout")]
    Timeout,
}