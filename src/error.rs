use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinkError {
    SeedLength,
    BufferTooSmall,
    FrameTooSmall,
    PayloadLengthOverflow,
    TruncatedFrame,
    InvalidFormat,
}

impl fmt::Display for PinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            PinkError::SeedLength => "seed must be exactly 9 bytes",
            PinkError::BufferTooSmall => "buffer too small",
            PinkError::FrameTooSmall => "frame too small",
            PinkError::PayloadLengthOverflow => "payload length overflow",
            PinkError::TruncatedFrame => "truncated frame",
            PinkError::InvalidFormat => "invalid PNK format",
        };
        f.write_str(msg)
    }
}
