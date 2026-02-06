use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::login::LoginContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub org_id: String,
}

#[derive(Debug, Deserialize)]
struct ListResponse {
    objects: Vec<Project>,
}

pub async fn list_projects(http: &Client, ctx: &LoginContext) -> Result<Vec<Project>> {
    let url = format!(
        "{}/v1/project?org_name={}",
        ctx.api_url.trim_end_matches('/'),
        encode(&ctx.login.org_name)
    );

    let response = http
        .get(&url)
        .bearer_auth(&ctx.login.api_key)
        .send()
        .await
        .context("failed to list projects")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("failed to list projects ({}): {}", status, body);
    }

    let list: ListResponse = response
        .json()
        .await
        .context("failed to parse projects response")?;

    Ok(list.objects)
}

pub async fn create_project(http: &Client, ctx: &LoginContext, name: &str) -> Result<Project> {
    let url = format!("{}/v1/project", ctx.api_url.trim_end_matches('/'));

    let body = serde_json::json!({ "name": name, "org_name": &ctx.login.org_name });

    let response = http
        .post(&url)
        .bearer_auth(&ctx.login.api_key)
        .json(&body)
        .send()
        .await
        .context("failed to create project")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("failed to create project ({}): {}", status, body);
    }

    response
        .json()
        .await
        .context("failed to parse create project response")
}

pub async fn delete_project(http: &Client, ctx: &LoginContext, project_id: &str) -> Result<()> {
    let url = format!(
        "{}/v1/project/{}",
        ctx.api_url.trim_end_matches('/'),
        project_id
    );

    let response = http
        .delete(&url)
        .bearer_auth(&ctx.login.api_key)
        .send()
        .await
        .context("failed to delete project")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("failed to delete project ({}): {}", status, body);
    }

    Ok(())
}

pub async fn get_project_by_name(
    http: &Client,
    ctx: &LoginContext,
    name: &str,
) -> Result<Option<Project>> {
    let url = format!(
        "{}/v1/project?org_name={}&name={}",
        ctx.api_url.trim_end_matches('/'),
        encode(&ctx.login.org_name),
        encode(name)
    );

    let response = http
        .get(&url)
        .bearer_auth(&ctx.login.api_key)
        .send()
        .await
        .context("failed to get project")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("failed to get project ({}): {}", status, body);
    }

    let list: ListResponse = response
        .json()
        .await
        .context("failed to parse project response")?;

    Ok(list.objects.into_iter().next())
}
