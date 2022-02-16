use aws_sdk_sfn::model::ExecutionStatus;
use std::fmt;

pub struct StateMachine {
    pub arn: String,
    pub name: String,
}

impl fmt::Display for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct StateMachineExecution {
    pub arn: String,
    pub name: String,
    pub status: ExecutionStatus, //Change to internal enum
    pub start_date: String,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl fmt::Display for StateMachineExecution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {}", self.name, self.start_date)
    }
}
