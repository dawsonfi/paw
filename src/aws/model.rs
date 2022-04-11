use chrono::{DateTime, SecondsFormat, Utc};
use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub struct StateMachine {
    pub arn: String,
    pub name: String,
}

impl fmt::Display for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, PartialEq, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct ExecutionInput {
    pub machine_arn: String,
    pub input: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::Utc;

    #[test]
    fn test_state_machine_execution_print_format() {
        let now = Utc::now();
        let execution = StateMachineExecution {
            arn: "dinosaur::arn".to_string(),
            machine_arn: "dinosaur_machine:arn".to_string(),
            name: "dinosaur".to_string(),
            start_date: now.clone(),
            input: Some("{}".to_string()),
            output: Some("{}".to_string()),
        };

        assert_eq!(
            format!("{}", execution),
            format!(
                "dinosaur : {}",
                now.to_rfc3339_opts(SecondsFormat::Secs, true)
            )
        );
    }
}
