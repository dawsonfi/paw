# paw
[![Current Crates.io Version](https://img.shields.io/crates/v/paw.svg)](https://crates.io/crates/paw)
[![Docs.rs](https://docs.rs/habitica-cli/badge.svg)](https://docs.rs/sfn-paw/)

Step Functions CLI Tool

# Usage

paw fetches the aws configuration from the `~/.aws/credentials` file, which should contain the following data:

* `region`
* `aws_access_key_id`
* `aws_secret_access_key`

for instructions see [AWS Configuration and credential file settings](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)

## Retry Failed Executions

1 - Select `Retry Failed Executions Action`

![Retry Failed Executions](https://raw.githubusercontent.com/dawsonfi/paw/assets/retry_failed_executions_1.png)

2 - Select the desired Step Functions Machine

![Retry Failed Executions](https://raw.githubusercontent.com/dawsonfi/paw/assets/retry_failed_executions_2.png)

3 - Input the Start Date (or leave it blank)

![Retry Failed Executions](https://raw.githubusercontent.com/dawsonfi/paw/assets/retry_failed_executions_3.png)

4 - Input the End Date (or leave it blank)

![Retry Failed Executions](https://raw.githubusercontent.com/dawsonfi/paw/assets/retry_failed_executions_4.png)

5 - Unmark any execution that you don't want to retry

![Retry Failed Executions](https://raw.githubusercontent.com/dawsonfi/paw/assets/retry_failed_executions_5.png)

6 - Press enter to retry selected executions

![Retry Failed Executions](https://raw.githubusercontent.com/dawsonfi/paw/assets/retry_failed_executions_6.png)