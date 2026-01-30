/// Represents the timeframe/interval of candlestick data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    M1,   // 1 minute
    M5,   // 5 minutes
    M15,  // 15 minutes
    M30,  // 30 minutes
    H1,   // 1 hour
    H4,   // 4 hours
    D1,   // 1 day
    W1,   // 1 week
}

impl Timeframe {
    /// Returns the duration of this timeframe in seconds
    pub fn to_seconds(&self) -> u64 {
        match self {
            Timeframe::M1 => 60,
            Timeframe::M5 => 5 * 60,
            Timeframe::M15 => 15 * 60,
            Timeframe::M30 => 30 * 60,
            Timeframe::H1 => 60 * 60,
            Timeframe::H4 => 4 * 60 * 60,
            Timeframe::D1 => 24 * 60 * 60,
            Timeframe::W1 => 7 * 24 * 60 * 60,
        }
    }

    /// Returns the duration of this timeframe in minutes
    pub fn to_minutes(&self) -> u64 {
        self.to_seconds() / 60
    }

    /// Returns a human-readable string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1d",
            Timeframe::W1 => "1w",
        }
    }

    /// Parses a timeframe string like "1m", "5m", "1h".
    pub fn from_str(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Timeframe {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "1m" => Ok(Timeframe::M1),
            "5m" => Ok(Timeframe::M5),
            "15m" => Ok(Timeframe::M15),
            "30m" => Ok(Timeframe::M30),
            "1h" => Ok(Timeframe::H1),
            "4h" => Ok(Timeframe::H4),
            "1d" => Ok(Timeframe::D1),
            "1w" => Ok(Timeframe::W1),
            _ => Err(()),
        }
    }
}
