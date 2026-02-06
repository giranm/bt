use std::io::IsTerminal;

use anyhow::{bail, Result};
use reqwest::Client;

use crate::login::LoginContext;
use crate::ui;

use super::api;

pub async fn run(http: &Client, ctx: &LoginContext, name: Option<&str>) -> Result<()> {
    let project_name = match name {
        Some(n) => {
            // Check if project exists
            if api::get_project_by_name(http, ctx, n).await?.is_none() {
                // Offer to create
                if !std::io::stdin().is_terminal() {
                    bail!("project '{}' not found", n);
                }

                let create = dialoguer::Confirm::new()
                    .with_prompt(format!("Project '{}' not found. Create it?", n))
                    .default(false)
                    .interact()?;

                if create {
                    api::create_project(http, ctx, n).await?;
                } else {
                    bail!("project '{}' not found", n);
                }
            }
            n.to_string()
        }
        None => select_project_interactive(http, ctx).await?,
    };

    ui::print_env_export(
        "BRAINTRUST_DEFAULT_PROJECT",
        &project_name,
        &format!("Switched to {}", project_name),
    );
    Ok(())
}

pub async fn select_project_interactive(http: &Client, ctx: &LoginContext) -> Result<String> {
    let mut projects = api::list_projects(http, ctx).await?;
    if projects.is_empty() {
        bail!("no projects found");
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();

    let selection = ui::fuzzy_select("Select project", &names)?;
    Ok(projects[selection].name.clone())
}
