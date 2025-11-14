//! Connascence types

/// Type of connascence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnascenceType {
    Name,
    Type,
    Meaning,
    Position,
    Algorithm,
}

impl std::fmt::Display for ConnascenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnascenceType::Name => write!(f, "Name"),
            ConnascenceType::Type => write!(f, "Type"),
            ConnascenceType::Meaning => write!(f, "Meaning"),
            ConnascenceType::Position => write!(f, "Position"),
            ConnascenceType::Algorithm => write!(f, "Algorithm"),
        }
    }
}

/// Strength of connascence (1-5)
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Strength {
    VeryLow = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    VeryHigh = 5,
}

impl std::fmt::Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strength::VeryLow => write!(f, "VeryLow"),
            Strength::Low => write!(f, "Low"),
            Strength::Medium => write!(f, "Medium"),
            Strength::High => write!(f, "High"),
            Strength::VeryHigh => write!(f, "VeryHigh"),
        }
    }
}
