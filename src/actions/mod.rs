use async_trait::async_trait;
use aws_sdk_sfn::Error;
use crate::actions::failed_executions::RetryFailedExecution;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub mod failed_executions;

#[async_trait]
pub trait StepFunctionsAction: Display {
    async fn execute(&self) -> Result<(), Error>;

    fn name(&self) -> String {
        "Invalid Action".to_string() 
    }    

    fn to_string(&self) -> String {
        self.name()
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.name())
    }

}

pub fn get_actions() -> Vec<Box<dyn StepFunctionsAction>> {
    vec![Box::new(RetryFailedExecution::new())]
}