use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiFeedbackSurface {
    Score,
    Recommendation,
    Forecast,
}

impl AiFeedbackSurface {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiFeedbackSurface::Score => "score",
            AiFeedbackSurface::Recommendation => "recommendation",
            AiFeedbackSurface::Forecast => "forecast",
        }
    }
}

impl fmt::Display for AiFeedbackSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AiFeedbackSurface {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "score" => Ok(AiFeedbackSurface::Score),
            "recommendation" => Ok(AiFeedbackSurface::Recommendation),
            "forecast" => Ok(AiFeedbackSurface::Forecast),
            other => Err(format!("unsupported AI feedback surface: {other}")),
        }
    }
}

impl FromStr for AiFeedbackSurface {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiFeedbackSentiment {
    Up,
    Down,
}

impl AiFeedbackSentiment {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiFeedbackSentiment::Up => "up",
            AiFeedbackSentiment::Down => "down",
        }
    }
}

impl fmt::Display for AiFeedbackSentiment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AiFeedbackSentiment {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "up" => Ok(AiFeedbackSentiment::Up),
            "down" => Ok(AiFeedbackSentiment::Down),
            other => Err(format!("unsupported AI feedback sentiment: {other}")),
        }
    }
}

impl FromStr for AiFeedbackSentiment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFeedback {
    pub id: i64,
    pub surface: AiFeedbackSurface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub sentiment: AiFeedbackSentiment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub prompt_snapshot: String,
    pub context_snapshot: Value,
    pub created_at: String,
    #[serde(default = "default_anonymized")]
    pub anonymized: bool,
}

fn default_anonymized() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFeedbackCreate {
    pub surface: AiFeedbackSurface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub sentiment: AiFeedbackSentiment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub prompt_snapshot: String,
    pub context_snapshot: Value,
    #[serde(default = "default_anonymized")]
    pub anonymized: bool,
}
