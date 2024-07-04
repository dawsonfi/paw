use crate::aws::model::{ExecutionInput, StateMachine, StateMachineExecution};
use aws_sdk_sfn::Error;
use chrono::{DateTime, TimeZone, Utc};
#[double]
use external_client::StepFunctionsClient;
use mockall_double::double;

#[allow(dead_code)]
mod external_client {
    use aws_config::from_env;
    use aws_sdk_sfn::{
        error::{
            DescribeExecutionError, ListExecutionsError, ListStateMachinesError,
            StartExecutionError,
        },
        model::ExecutionStatus,
        output::{
            DescribeExecutionOutput, ListExecutionsOutput, ListStateMachinesOutput,
            StartExecutionOutput,
        },
        Client,
    };
    use aws_smithy_http::result::SdkError;

    pub struct StepFunctionsClient {
        pub client: Client,
    }

    #[cfg_attr(test, mockall::automock)]
    impl StepFunctionsClient {
        pub async fn new() -> Self {
            let config = from_env().load().await;
            StepFunctionsClient {
                client: Client::new(&config),
            }
        }

        pub async fn list_state_machines(
            &self,
        ) -> Result<ListStateMachinesOutput, SdkError<ListStateMachinesError>> {
            self.client.list_state_machines().send().await
        }

        pub async fn list_failed_executions(
            &self,
            state_machine_arn: String,
            next_token: Option<String>,
        ) -> Result<ListExecutionsOutput, SdkError<ListExecutionsError>> {
            let mut req = self
                .client
                .list_executions()
                .state_machine_arn(state_machine_arn)
                .max_results(1000)
                .status_filter(ExecutionStatus::Failed);

            if next_token.is_some() {
                req = req.next_token(next_token.unwrap());
            }

            req.send().await
        }

        pub async fn describe_execution(
            &self,
            execution_arn: String,
        ) -> Result<DescribeExecutionOutput, SdkError<DescribeExecutionError>> {
            self.client
                .describe_execution()
                .execution_arn(execution_arn)
                .send()
                .await
        }

        pub async fn start_execution(
            &self,
            state_machine_arn: String,
            input: String,
        ) -> Result<StartExecutionOutput, SdkError<StartExecutionError>> {
            self.client
                .start_execution()
                .state_machine_arn(state_machine_arn)
                .input(input)
                .send()
                .await
        }
    }
}

pub struct StepFunctionsMachine {
    client: StepFunctionsClient,
}

impl StepFunctionsMachine {
    pub async fn new() -> StepFunctionsMachine {
        StepFunctionsMachine {
            client: StepFunctionsClient::new().await,
        }
    }

    pub async fn list_machines(&self) -> Result<Vec<StateMachine>, Error> {
        let machines = self.client.list_state_machines().await?;
        let machine_names = machines
            .state_machines
            .unwrap_or_default()
            .into_iter()
            .map(|machine| StateMachine {
                arn: machine.state_machine_arn.unwrap(),
                name: machine.name.unwrap(),
            })
            .collect::<Vec<StateMachine>>();

        Ok(machine_names)
    }

    pub async fn list_failed_executions(
        &self,
        machine: &StateMachine,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<StateMachineExecution>, Error> {
        let mut executions: Vec<StateMachineExecution> = vec![];
        let mut next_token: Option<String> = None;

        loop {
            let raw_executions = self
                .client
                .list_failed_executions(machine.arn.clone(), next_token)
                .await?;

            next_token = raw_executions.next_token;

            let mut partial_executions = raw_executions
                .executions
                .unwrap_or_default()
                .into_iter()
                .map(|execution| StateMachineExecution {
                    arn: execution.execution_arn.unwrap(),
                    machine_arn: execution.state_machine_arn.unwrap(),
                    name: execution.name.unwrap(),
                    start_date: StepFunctionsMachine::convert_date_time(
                        execution.start_date.unwrap(),
                    ),
                    input: Option::None,
                    output: Option::None,
                })
                .filter(|execution| {
                    let start = start_date.unwrap_or(Utc::now());
                    let end = end_date.unwrap_or(Utc::now());
                    let execution_time = execution.start_date;

                    if start_date.is_some() && end_date.is_some() {
                        return start <= execution_time && end >= execution_time;
                    } else if start_date.is_some() {
                        return start <= execution_time;
                    } else if end_date.is_some() {
                        return end >= execution_time;
                    }

                    true
                })
                .collect::<Vec<StateMachineExecution>>();

            executions.append(&mut partial_executions);

            if next_token.is_none() {
                break;
            }
        }

        Ok(executions)
    }

    pub async fn describe_execution(
        &self,
        execution_arn: String,
    ) -> Result<StateMachineExecution, Error> {
        let raw_execution = self.client.describe_execution(execution_arn).await?;

        let execution = StateMachineExecution {
            arn: raw_execution.execution_arn.unwrap(),
            machine_arn: raw_execution.state_machine_arn.unwrap(),
            name: raw_execution.name.unwrap(),
            start_date: StepFunctionsMachine::convert_date_time(raw_execution.start_date.unwrap()),
            input: raw_execution.input,
            output: raw_execution.output,
        };

        Ok(execution)
    }

    pub async fn start_execution(&self, input: ExecutionInput) -> Result<(), Error> {
        self.client
            .start_execution(input.machine_arn, input.input)
            .await?;

        Ok(())
    }

    fn convert_date_time(date: aws_smithy_types::DateTime) -> DateTime<Utc> {
        Utc.from_utc_datetime(
            &DateTime::from_timestamp(date.secs(), date.subsec_nanos())
                .unwrap()
                .naive_utc(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_sfn::model::execution_list_item::Builder as ExecutionListItemBuilder;
    use aws_sdk_sfn::model::state_machine_list_item::Builder as StateMachineListItemBuilder;
    use aws_sdk_sfn::output::describe_execution_output::Builder as DescribeExecutionBuilder;
    use aws_sdk_sfn::output::list_executions_output::Builder as ListExecutionsBuilder;
    use aws_sdk_sfn::output::list_state_machines_output::Builder as ListStateMachinesBuilder;
    use aws_sdk_sfn::output::start_execution_output::Builder as StartExecutioBuilder;
    use aws_smithy_types::DateTime;

    use mockall::predicate::eq;

    #[tokio::test]
    async fn should_return_state_machines() {
        let mut result = Some(Ok(ListStateMachinesBuilder::default()
            .state_machines(
                StateMachineListItemBuilder::default()
                    .state_machine_arn("dinosaur_machine::arn")
                    .name("dinosaur_machine")
                    .build(),
            )
            .build()));
        let mut mock_client = StepFunctionsClient::default();
        mock_client
            .expect_list_state_machines()
            .with()
            .times(1)
            .returning(move || result.take().unwrap());

        let machine = StepFunctionsMachine {
            client: mock_client,
        };

        assert_eq!(
            machine.list_machines().await.unwrap(),
            vec![StateMachine {
                arn: "dinosaur_machine::arn".to_string(),
                name: "dinosaur_machine".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn should_return_empty_state_machines() {
        let mut result = Some(Ok(ListStateMachinesBuilder::default()
            .set_state_machines(None)
            .build()));
        let mut mock_client = StepFunctionsClient::default();
        mock_client
            .expect_list_state_machines()
            .with()
            .times(1)
            .returning(move || result.take().unwrap());

        let machine = StepFunctionsMachine {
            client: mock_client,
        };

        assert_eq!(machine.list_machines().await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn should_return_failed_executions_inside_date_range() {
        let utc_now = DateTime::from_secs(Utc::now().timestamp());
        let mut result = Some(Ok(ListExecutionsBuilder::default()
            .executions(
                ExecutionListItemBuilder::default()
                    .execution_arn("dinosaur::arn::exec")
                    .state_machine_arn("dinosaur::arn")
                    .name("Execution")
                    .start_date(utc_now)
                    .build(),
            )
            .build()));
        let mut mock_client = StepFunctionsClient::default();
        mock_client
            .expect_list_failed_executions()
            .with(eq("dinosaur::arn".to_string()), eq(None))
            .times(1)
            .returning(move |_state_machine_arn, _next_token| result.take().unwrap());

        let machine = StepFunctionsMachine {
            client: mock_client,
        };

        let state_machine = StateMachine {
            arn: "dinosaur::arn".to_string(),
            name: "dinosaur".to_string(),
        };
        let failed_executions = machine
            .list_failed_executions(&state_machine, None, None)
            .await
            .unwrap();

        assert_eq!(
            failed_executions,
            vec![StateMachineExecution {
                arn: "dinosaur::arn::exec".to_string(),
                machine_arn: "dinosaur::arn".to_string(),
                name: "Execution".to_string(),
                start_date: StepFunctionsMachine::convert_date_time(utc_now),
                input: None,
                output: None
            }]
        )
    }

    #[tokio::test]
    async fn should_return_empty_failed_executions() {
        let mut result = Some(Ok(ListExecutionsBuilder::default()
            .set_executions(None)
            .build()));
        let mut mock_client = StepFunctionsClient::default();
        mock_client
            .expect_list_failed_executions()
            .with(eq("dinosaur::arn".to_string()), eq(None))
            .times(1)
            .returning(move |_state_machine_arn, _next_token| result.take().unwrap());

        let machine = StepFunctionsMachine {
            client: mock_client,
        };

        let state_machine = StateMachine {
            arn: "dinosaur::arn".to_string(),
            name: "dinosaur".to_string(),
        };
        let failed_executions = machine
            .list_failed_executions(&state_machine, None, None)
            .await
            .unwrap();

        assert_eq!(failed_executions, vec![])
    }

    #[tokio::test]
    async fn should_return_execution() {
        let utc = DateTime::from_secs(Utc::now().timestamp());
        let mut result = Some(Ok(DescribeExecutionBuilder::default()
            .execution_arn("dinosaur::arn")
            .state_machine_arn("dinousar::machine")
            .name("dinosaur")
            .start_date(utc)
            .input("{'batata': 'frita'}")
            .output("{'body': 'delicia'}")
            .build()));

        let mut mock_client = StepFunctionsClient::default();
        mock_client
            .expect_describe_execution()
            .with(eq("dinousar::arn".to_string()))
            .times(1)
            .returning(move |_machine_arn| result.take().unwrap());

        let machine = StepFunctionsMachine {
            client: mock_client,
        };

        let execution = machine
            .describe_execution("dinousar::arn".to_string())
            .await
            .unwrap();

        assert_eq!(
            execution,
            StateMachineExecution {
                arn: "dinosaur::arn".to_string(),
                machine_arn: "dinousar::machine".to_string(),
                name: "dinosaur".to_string(),
                start_date: StepFunctionsMachine::convert_date_time(utc),
                input: Some("{'batata': 'frita'}".to_string()),
                output: Some("{'body': 'delicia'}".to_string())
            }
        )
    }

    #[tokio::test]
    async fn should_start_execution() {
        let mut result = Some(Ok(StartExecutioBuilder::default()
            .execution_arn("dinousar::arn")
            .start_date(DateTime::from_secs(Utc::now().timestamp()))
            .build()));
        let mut mock_client = StepFunctionsClient::default();
        mock_client
            .expect_start_execution()
            .with(
                eq("dinosaur::arn".to_string()),
                eq("{'batata': 'frita'}".to_string()),
            )
            .times(1)
            .returning(move |_machine_arn, _input| result.take().unwrap());

        let machine = StepFunctionsMachine {
            client: mock_client,
        };

        machine
            .start_execution(ExecutionInput {
                machine_arn: "dinosaur::arn".to_string(),
                input: "{'batata': 'frita'}".to_string(),
            })
            .await
            .unwrap();
    }
}
