use chrono::{DateTime, SecondsFormat, Utc};
use std::fmt;

#[derive(Clone)]
pub struct StateMachine {
    pub arn: String,
    pub name: String,
}

impl fmt::Display for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone)]
pub struct StateMachineExecution {
    pub arn: String,
    pub machine_arn: String,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl fmt::Display for StateMachineExecution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} : {}",
            self.name,
            self.start_date.to_rfc3339_opts(SecondsFormat::Secs, true)
        )
    }
}

#[derive(Clone)]
pub struct ExecutionInput {
    pub machine_arn: String,
    pub input: String,
}
