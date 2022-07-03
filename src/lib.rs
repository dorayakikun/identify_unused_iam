// Copyright (c) 2022 Tomohide Takao
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

// cf. https://aws.amazon.com/blogs/security/review-last-accessed-information-to-identify-unused-ec2-iam-and-lambda-permissions-and-tighten-access-for-iam-roles/

use aws_sdk_iam::Client;
use eyre::{Context, Result};
use std::io::{stdout, BufWriter, Write};
use structopt::{clap, StructOpt};
use unused_policies::{fetch_unused_policies, print_delete_policies_scripts};
use unused_roles::{
    fetch_role_policies_by_unused_role, fetch_unused_roles, print_delete_roles_scripts,
};

mod unused_policies;
mod unused_roles;

#[derive(Debug, StructOpt)]
#[structopt(name = "identify_unused_iam")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(subcommand)]
    pub sub: Sub,
}

#[derive(Debug, StructOpt)]
pub enum Sub {
    #[structopt(name = "list-unused-roles", about = "Lists the unused IAM roles")]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    ListUnusedRoles {
        #[structopt(long = "path-prefix")]
        path_prefix: Option<String>,
        #[structopt(long = "last-accessed")]
        last_accessed: Option<u64>,
        #[structopt(long = "include-service-roles")]
        include_service_roles: bool,
        #[structopt(long = "exclude-last-accessed-none")]
        exclude_last_accessed_none: bool,
    },
    #[structopt(
        name = "print-delete-unused-roles-scripts",
        about = "Prints scripts that delete the unused IAM roles"
    )]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    PrintDeleteUnusedRolesScripts {
        #[structopt(long = "path-prefix")]
        path_prefix: Option<String>,
        #[structopt(long = "last-accessed")]
        last_accessed: Option<u64>,
        #[structopt(long = "include-service-roles")]
        include_service_roles: bool,
        #[structopt(long = "exclude-last-accessed-none")]
        exclude_last_accessed_none: bool,
    },

    #[structopt(name = "list-unused-policies", about = "Lists the unused policies")]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    ListUnusedPolicies {
        #[structopt(long = "path-prefix")]
        path_prefix: Option<String>,
    },

    #[structopt(
        name = "print-delete-unused-policies-scripts",
        about = "Prints scripts that delete the unused policies"
    )]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    PrintDeleteUnusedPoliciesScripts {
        #[structopt(long = "path-prefix")]
        path_prefix: Option<String>,
    },
}

pub async fn run() -> Result<()> {
    let shared_config = aws_config::from_env().load().await;
    let client = Client::new(&shared_config);

    let opt = Opt::from_args();
    match opt.sub {
        Sub::ListUnusedRoles {
            path_prefix,
            last_accessed,
            include_service_roles,
            exclude_last_accessed_none,
        } => {
            let unused_roles = fetch_unused_roles(
                &client,
                &path_prefix,
                &last_accessed,
                include_service_roles,
                exclude_last_accessed_none,
            )
            .await
            .wrap_err_with(|| {
                format!(
                    "Failed to fetch unused roles when lists unused roles.\nclient is {:#?}.",
                    &client
                )
            })?;

            let mut wtr = csv::Writer::from_writer(stdout());

            for ur in &unused_roles {
                wtr.serialize(ur).wrap_err_with(|| {
                    format!("Failed to serialize.\nunused_role is {:#?}.", &ur)
                })?;
            }
            wtr.flush()
                .wrap_err("Failed to flush when lists unsued roles.")?;
        }
        Sub::PrintDeleteUnusedRolesScripts {
            path_prefix,
            last_accessed,
            include_service_roles,
            exclude_last_accessed_none,
        } => {
            let unused_roles = fetch_unused_roles(&client, &path_prefix, &last_accessed, include_service_roles, exclude_last_accessed_none).await.wrap_err_with(
                || format!("Failed to fetch unused roles when prints delete unused roles scripts.\nclient is {:#?}", &client)
            )?;

            let role_policies_by_unused_role =
                fetch_role_policies_by_unused_role(&client, &unused_roles).await
                .wrap_err_with(|| format!("Failed to fetch unused roles-policies by unused role when prints delete unused roles scripts.\nclient is {:#?}.", &client))?;

            let out = stdout();
            let mut out = BufWriter::new(out.lock());

            print_delete_roles_scripts(&mut out, &role_policies_by_unused_role).wrap_err_with(
                || {
                    format!(
                        "Failed to print delete roles scripts.\nrole policies are {:#?}.",
                        &role_policies_by_unused_role
                    )
                },
            )?;

            out.flush()
                .wrap_err("Failed to flush when prints delete unused roles scripts.")?;
        }
        Sub::ListUnusedPolicies { path_prefix } => {
            let unused_policies = fetch_unused_policies(&client, &path_prefix)
                .await
                .wrap_err_with(|| {
                    format!(
                    "Failed to fetch unused policies when lists unused policies.\nclient is {:#?}",
                    &client
                )
                })?;

            let mut wtr = csv::Writer::from_writer(stdout());
            for up in &unused_policies {
                wtr.serialize(up).wrap_err_with(|| {
                    format!("Failed to serealize.\nunused policy is {:#?}.", &up)
                })?;
            }

            wtr.flush()
                .wrap_err("Failed to flush when lists unsued policies.")?;
        }
        Sub::PrintDeleteUnusedPoliciesScripts { path_prefix } => {
            let unused_policies = fetch_unused_policies(&client, &path_prefix).await.wrap_err_with(|| {
                format!(
                    "Failed to fetch unused policies when print delete unused policies scripts.\nclient is {:#?}",
                    &client
                )
            })?;

            let out = stdout();
            let mut out = BufWriter::new(out.lock());
            print_delete_policies_scripts(&mut out, &unused_policies).wrap_err_with(|| {
                format!(
                    "Failed to print delete unused policies scripts.\nunused policies are {:#?}",
                    &unused_policies
                )
            })?;

            out.flush()
                .wrap_err("Failed to flush when prints delete unused policies scripts.")?;
        }
    }
    Ok(())
}
