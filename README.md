# identify_unused_iam

## getting started

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ cargo install --git https://github.com/dorayakikun/identify_unused_iam
$ export AWS_ACCESS_KEY_ID=${YOUR_AWS_ACCESS_KEY_ID}
$ export AWS_SECRET_ACCESS_KEY=${YOUR_AWS_SECRET_ACCESS_KEY}
```

## commands

- list-unused-policies
- list-unused-roles
- print-delete-unused-policies-scripts
- print-delete-unused-roles-scripts

## What's the unused IAM role ?

The IAM role whose last activity is 90 days old.
(The IAM roles that do not have a last activity are excluded from the results. This is because IAM roles immediately after creation are not included.)

## What's the unused policies ?

A policy that is not attached to the IAM roles.
(It means that "attachement count == 0".)

## examples

```bash
$ identify_unused_iam
identify_unused_iam 0.1.0

USAGE:
    identify_unused_iam <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help                                    Prints this message or the help of the given subcommand(s)
    list-unused-policies                    Lists the unused policies
    list-unused-roles                       Lists the unused IAM roles
    print-delete-unused-policies-scripts    Prints scripts that delete the unused policies
    print-delete-unused-roles-scripts       Prints scripts that delete the unused IAM roles

$ identify_unused_iam list-unused-policies
AWSLambdaBasicExecutionRole-5fda812a-bc59-410f-aa05-ba3bdd677b93,arn:aws:iam::{YOUR_ACCOUNT_ID}:policy/service-role/AWSLambdaBasicExecutionRole-5fda812a-bc59-410f-aa05-ba3bdd677b93,/service-role/,2017-05-28T02:52:59Z,
AWSLambdaBasicExecutionRole-ca19ec60-8fa0-4734-b9cb-cbb51661432d,arn:aws:iam::{YOUR_ACCOUNT_ID}:policy/service-role/AWSLambdaBasicExecutionRole-ca19ec60-8fa0-4734-b9cb-cbb51661432d,/service-role/,2017-05-27T09:20:48Z,

$ identify_unused_iam list-unused-roles
role_name,arn,path,created_date,role_last_used,description
AWSServiceRoleForAWSCloud9,arn:aws:iam::{YOUR_ACCOUNT_ID}:role/aws-service-role/cloud9.amazonaws.com/AWSServiceRoleForAWSCloud9,/aws-service-role/cloud9.amazonaws.com/,2020-09-06T06:03:39Z,,Service linked role for AWS Cloud9
AWSServiceRoleForSupport,arn:aws:iam::{YOUR_ACCOUNT_ID}:role/aws-service-role/support.amazonaws.com/AWSServiceRoleForSupport,/aws-service-role/support.amazonaws.com/,2018-12-22T22:39:04Z,,"Enables resource access for AWS to provide billing, administrative and support services"
AWSServiceRoleForTrustedAdvisor,arn:aws:iam::{YOUR_ACCOUNT_ID}:role/aws-service-role/trustedadvisor.amazonaws.com/AWSServiceRoleForTrustedAdvisor,/aws-service-role/trustedadvisor.amazonaws.com/,2019-01-20T03:40:19Z,,"Access for the AWS Trusted Advisor Service to help reduce cost, increase performance, and improve security of your AWS environment."
LambdaRole,arn:aws:iam::{YOUR_ACCOUNT_ID}:role/service-role/LambdaRole,/service-role/,2017-05-28T02:52:58Z,,

$ identify_unused_iam print-delete-unused-policies-scripts
aws iam delete-policy --policy-arn 'arn:aws:iam::{YOUR_ACCOUNT_ID}:policy/service-role/AWSLambdaBasicExecutionRole-5fda812a-bc59-410f-aa05-ba3bdd677b93'
aws iam delete-policy --policy-arn 'arn:aws:iam::{YOUR_ACCOUNT_ID}:policy/service-role/AWSLambdaBasicExecutionRole-ca19ec60-8fa0-4734-b9cb-cbb51661432d'

$ identify_unused_iam print-delete-unused-roles-scripts
aws iam detach-role-policy --role-name 'LambdaRole' --policy-arn 'arn:aws:iam::{YOUR_ACCOUNT_ID}:policy/service-role/AWSLambdaCloudFormationExecutionRole-73a5f013-c4d4-4c9e-9fa1-e9fbd3aaef8b'
aws iam detach-role-policy --role-name 'LambdaRole' --policy-arn 'arn:aws:iam::{YOUR_ACCOUNT_ID}:policy/service-role/AWSLambdaBasicExecutionRole-5fda812a-bc59-410f-aa05-ba3bdd677b93'
aws iam delete-role --role-name 'LambdaRole'
```

## TODO

- `--profile` option
