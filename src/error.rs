use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
   InvalidLevel,
   UnknownType,
   UnreachableTarget,
   SourceDoesNotExists,
   Other(String)
}

impl ErrorKind {
    //This Function just for creating ErrorKind type from the Ruby code
    pub fn new_by_string(symbol: &str) -> ErrorKind {
        let symbol = symbol.to_lowercase();
        if &symbol == "invalidlevel" {
            ErrorKind::InvalidLevel
        } else if &symbol == "unkowntype" {
            ErrorKind::UnknownType
        } else if &symbol == "unreachabletarget" {
            ErrorKind::UnreachableTarget 
        } else if &symbol == "sourcedoesnotexists" {
            ErrorKind::SourceDoesNotExists
        } else {
            ErrorKind::Other(symbol)
        }
   }
}

impl Error for ErrorKind {
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match &self {
            ErrorKind::InvalidLevel => "Ordered compression level is wrong for this type.".to_owned(),
            ErrorKind::UnknownType => "Ordered compression type does not recognized".to_owned(),
            ErrorKind::UnreachableTarget => "The target does not reachable".to_owned(),
            ErrorKind::SourceDoesNotExists => "The source does not exists".to_owned(),
            ErrorKind::Other(x) => format!("{}",x)
        };
        write!(f, "{}",msg)
    }
}

#[derive(Debug)]
pub struct CompressionError {
    kind: ErrorKind,
}

impl Error for CompressionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Compression Error: {}", &self.kind)
    }
}

impl CompressionError {
    pub fn new(kind: ErrorKind) -> CompressionError {
        CompressionError {
            kind
        }
    }
}
