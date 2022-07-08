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
```

## TODO

- `--profile` option
