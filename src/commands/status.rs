use clap::Args;

use crate::{config::Config, output::Printer, workspace::WorkspaceInfo};

#[derive(Debug, Args)]
pub struct StatusArgs {}

pub fn run(
    _args: &StatusArgs,
    _config: &Config,
    printer: &Printer,
    repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    let workspaces = WorkspaceInfo::load_all(repo_root)?;

    let current = workspaces.iter().find(|w| w.is_current);
    let others: Vec<_> = workspaces.iter().filter(|w| !w.is_current).collect();

    if let Some(ws) = current {
        let name = printer.domain_name(&ws.name, true);
        println!("Current domain: {name}");
        println!("Path:           {}", printer.path_text(&ws.display_path()));

        if let Some(cid) = &ws.change_id {
            println!("Change ID:      {}", printer.change_id_text(cid));
        }

        if let Some(desc) = &ws.description {
            let d = if desc.is_empty() { "(empty)" } else { desc.as_str() };
            println!("Description:    {d}");
        }
    } else {
        printer.warning("no current workspace detected");
    }

    if !others.is_empty() {
        println!();
        println!("Other domains ({}):", others.len());
        for ws in &others {
            let name = printer.domain_name(&ws.name, false);
            let status = if ws.is_stale {
                printer.stale_text("stale")
            } else {
                "active".to_owned()
            };
            println!("  {:<20} {:<40} {}", name, printer.path_text(&ws.display_path()), status);
        }
    }

    Ok(())
}
