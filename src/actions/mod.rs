use crate::actions::failed_executions::RetryFailedExecution;
use async_trait::async_trait;
use aws_sdk_sfn::Error;
use std::fmt::Display;

pub mod failed_executions;

#[async_trait]
pub trait StepFunctionsAction: Display {
    async fn execute(&self) -> Result<(), Error>;

    fn name(&self) -> String {
        "Invalid Action".to_string()
    }
}

pub fn get_actions() -> Vec<Box<dyn StepFunctionsAction>> {
    vec![Box::new(RetryFailedExecution::new())]
}

#[cfg(test)]
mod tests {
    use std::{fmt, fmt::Formatter};
    use super::*;

    struct TestAction {}

    #[async_trait]
    impl StepFunctionsAction for TestAction {
        async fn execute(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    impl Display for TestAction {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "Test: {}", self.name())
        }
    }

    #[test]
    fn should_return_default_name() {
        let test_action = TestAction {};

        assert_eq!(test_action.name(), "Invalid Action".to_string());
    }

    #[test]
    fn should_return_name_on_display() {
        let test_action = TestAction {};

        assert_eq!(
            format!("{}", test_action),
            "Test: Invalid Action".to_string()
        );
    }
}
