// Copyright (c) 2022 Tomohide Takao
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use aws_sdk_iam::model::RoleLastUsed;
use aws_sdk_iam::output::{GetRoleOutput, ListAttachedRolePoliciesOutput, ListRolesOutput};
use aws_sdk_iam::types::DateTime;
use aws_sdk_iam::Client;
use eyre::{Result, WrapErr};
use futures::stream::{FuturesUnordered, StreamExt};
use regex::Regex;
use serde::Serialize;
use std::io::{BufWriter, Write};
use time::OffsetDateTime;

#[derive(Debug, Serialize)]
pub struct UnusedRole {
    pub role_name: Option<String>,
    pub arn: Option<String>,
    pub path: Option<String>,
    #[serde(with = "date_format")]
    pub created_date: Option<DateTime>,
    #[serde(with = "role_last_used_format")]
    pub role_last_used: Option<RoleLastUsed>,
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

mod role_last_used_format {
    use aws_sdk_iam::model::RoleLastUsed;
    // HACK: Import by myself because it isn't reexported.
    use aws_smithy_types::date_time::Format;
    use serde::{self, Serializer};

    pub fn serialize<S>(date: &Option<RoleLastUsed>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *date {
            None => serializer.serialize_none(),
            Some(ref role_last_used) => match role_last_used.last_used_date {
                None => serializer.serialize_none(),
                Some(ref last_used_date) => {
                    let s = format!("{}", last_used_date.fmt(Format::DateTime).unwrap());
                    serializer.serialize_some(&s)
                }
            },
        }
    }
}

pub async fn fetch_unused_roles(
    client: &Client,
    path_prefix: &Option<String>,
    last_accessed: &Option<u64>,
    include_service_roles: bool,
    exclude_last_accessed_none: bool,
) -> Result<Vec<UnusedRole>> {
    let roles = fetch_roles(client, path_prefix).await?;

    let role_names = roles
        .roles
        .unwrap_or(vec![])
        .into_iter()
        .filter_map(|r| r.role_name.clone())
        .collect::<Vec<String>>();

    let roles = fetch_role_details(client, &role_names).await;

    let mut errors = vec![];

    let unused_roles = roles
        .into_iter()
        .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
        .filter(|g| is_unused_role(g, last_accessed, exclude_last_accessed_none))
        .filter(|g| {
            if !include_service_roles {
                if let Some(g) = g.role() {
                    if let Some(arn) = g.arn() {
                        let re = Regex::new(
                            r"^arn:aws:iam::\d{12}:role/(aws-service-role|service-role)/*",
                        )
                        .unwrap(); // TODO
                        return !re.is_match(arn);
                    }
                }
            }
            true
        })
        .filter_map(|g| g.role)
        .map(|r| UnusedRole {
            role_name: r.role_name,
            arn: r.arn,
            path: r.path,
            created_date: r.create_date,
            role_last_used: r.role_last_used,
            description: r.description,
        })
        .collect::<Vec<_>>();

    if errors.len() > 0 {
        return Err(eyre::eyre!(
            "(fetch_unused_roles)\nerrors are {:#?}.",
            errors
        ));
    }

    Ok(unused_roles)
}

async fn fetch_roles(client: &Client, path_prefix: &Option<String>) -> Result<ListRolesOutput> {
    client
        .list_roles()
        .set_path_prefix(path_prefix.clone())
        .send()
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to call the list-roles API.\npath_prefix is {:#?}.",
                path_prefix
            )
        })
}

async fn fetch_role_details(
    client: &Client,
    role_names: &Vec<String>,
) -> Vec<Result<GetRoleOutput>> {
    role_names
        .into_iter()
        .map(|r| {
            let client = client.clone();
            async move {
                client
                    .get_role()
                    .role_name(r.clone())
                    .send()
                    .await
                    .wrap_err_with(|| {
                        format!("Failed to call the get-role API. role-name is {}", &r)
                    })
            }
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
}

fn is_unused_role(
    g: &GetRoleOutput,
    last_accessed: &Option<u64>,
    exclude_last_accessed_none: bool,
) -> bool {
    if let Some(role) = g.role() {
        if let Some(last_used) = role.role_last_used() {
            if let Some(last_used_date) = last_used.last_used_date() {
                return is_unused(last_accessed, last_used_date);
            } else {
                // HACK: Even if the last activity is None, there are cases where RoleLastUsed is Some, so processing is also applied here.
                if exclude_last_accessed_none {
                    return false;
                }
                return true;
            }
        } else {
            if exclude_last_accessed_none {
                return false;
            }
            return true;
        }
    }
    return false;
}

fn is_unused(last_accessed: &Option<u64>, last_used_date: &DateTime) -> bool {
    let last_used_date = OffsetDateTime::from_unix_timestamp_nanos(last_used_date.as_nanos())
        .expect("Convert nanos to datetime");
    let now = OffsetDateTime::now_utc();
    let diff = now - last_used_date;

    if diff.whole_days() >= last_accessed.unwrap_or(90) as i64 {
        return true;
    }
    false
}

pub async fn fetch_role_policies_by_unused_role(
    client: &Client,
    unused_roles: &Vec<UnusedRole>,
) -> Result<Vec<(Option<String>, ListAttachedRolePoliciesOutput)>> {
    let rpur = unused_roles
        .into_iter()
        .map(|u| async move {
            let role_policies = client
                .list_attached_role_policies()
                .set_role_name(u.role_name.clone())
                .send()
                .await
                .wrap_err_with(|| {
                    format!(
                        "Failed to call the list-attached-role-policies API.\n role_name is {:#?}",
                        u.role_name
                    )
                });
            (u.role_name.clone(), role_policies)
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    let mut errors = vec![];

    let mut new_rpur = vec![];
    for (k, v) in rpur {
        match v {
            Ok(l) => new_rpur.push((k, l)),
            Err(e) => errors.push(e),
        }
    }

    if errors.len() > 0 {
        return Err(eyre::eyre!(
            "(fetch_role_policies_by_unused_role)\nerrors are {:#?}.",
            errors
        ));
    }

    Ok(new_rpur)
}

pub fn print_delete_roles_scripts<W: Write>(
    w: &mut BufWriter<W>,
    urpr: &Vec<(Option<String>, ListAttachedRolePoliciesOutput)>,
) -> Result<()> {
    for (k, v) in urpr {
        if let Some(role_name) = k {
            if let Some(attached_policies) = v.attached_policies() {
                for a in attached_policies {
                    if let Some(policy_arn) = a.policy_arn() {
                        writeln!(w, "aws iam detach-role-policy --role-name '{role_name}' --policy-arn '{policy_arn}'")?;
                    }
                }
            }
            writeln!(w, "aws iam delete-role --role-name '{role_name}'")?;
            writeln!(w, "")?;
        }
    }
    Ok(())
}
