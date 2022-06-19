// Copyright (c) 2022 Tomohide Takao
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use aws_sdk_iam::model::Policy;
use aws_sdk_iam::output::{GetPolicyOutput, ListPoliciesOutput};
use aws_sdk_iam::types::DateTime;
use aws_sdk_iam::Client;
use eyre::{Result, WrapErr};
use futures::stream::{FuturesUnordered, StreamExt};
use serde::Serialize;
use std::io::{BufWriter, Write};

#[derive(Debug, Serialize)]
pub struct UnusedPolicy {
    pub policy_name: Option<String>,
    pub arn: Option<String>,
    pub path: Option<String>,
    #[serde(with = "date_format")]
    pub create_date: Option<DateTime>,
    pub description: Option<String>,
}

mod date_format {
    use aws_sdk_iam::types::DateTime;
    // HACK: Import by myself because it isn't reexported.
    use aws_smithy_types::date_time::Format;
    use serde::{self, Serializer};

    pub fn serialize<S>(date: &Option<DateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *date {
            None => serializer.serialize_none(),
            Some(ref date) => {
                let s = format!("{}", date.fmt(Format::DateTime).unwrap());
                serializer.serialize_some(&s)
            }
        }
    }
}

pub async fn fetch_unused_policies(
    client: &Client,
    path_prefix: &Option<String>,
) -> Result<Vec<UnusedPolicy>> {
    let ret = fetch_policies(client, path_prefix).await?;

    let unused_policies = if let Some(policies) = ret.policies() {
        let policies = fetch_policy_details(client, policies).await;

        let mut errors = vec![];

        let unused_policies = policies
            .into_iter()
            .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
            .filter_map(|g| g.policy)
            .filter(|p| !is_attached(p))
            .map(|p| UnusedPolicy {
                policy_name: p.policy_name,
                arn: p.arn,
                path: p.path,
                create_date: p.create_date,
                description: p.description,
            })
            .collect::<Vec<_>>();

        if errors.len() > 0 {
            return Err(eyre::eyre!(
                "(fetch_unused_policies)\nerrors are {:#?}",
                errors
            ));
        }

        unused_policies
    } else {
        vec![]
    };
    Ok(unused_policies)
}

async fn fetch_policies(
    client: &Client,
    path_prefix: &Option<String>,
) -> Result<ListPoliciesOutput> {
    client
        .list_policies()
        .set_path_prefix(path_prefix.clone())
        .send()
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to call the list-policies API.\npath_prefix is {:#?}.",
                path_prefix
            )
        })
}

async fn fetch_policy_details(
    client: &Client,
    policies: &[Policy],
) -> Vec<Result<GetPolicyOutput>> {
    policies
        .into_iter()
        .map(|p| {
            let client = client.clone();
            async move {
                client
                    .get_policy()
                    .set_policy_arn(p.arn.clone())
                    .send()
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "Failed to call the get-policy API. policy-arn is {:#?}",
                            &p.arn
                        )
                    })
            }
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
}

fn is_attached(p: &Policy) -> bool {
    if let Some(a) = p.attachment_count() {
        if a >= 0 {
            return false;
        } else {
            return true;
        }
    }
    false
}

pub fn print_delete_policies_scripts<W: Write>(
    w: &mut BufWriter<W>,
    unused_policies: &Vec<UnusedPolicy>,
) -> Result<()> {
    for up in unused_policies {
        if let Some(arn_name) = &up.arn {
            writeln!(w, "aws iam delete-policy --policy-arn '{}'", arn_name)?;
        }
    }
    Ok(())
}
