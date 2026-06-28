use clap::Args;
use comfy_table::{Attribute, Cell, Color, Table};

use crate::{config::Config, output::Printer, workspace::WorkspaceInfo};

#[derive(Debug, Args)]
pub struct ListArgs {
    /// 出力形式: table / simple / json / porcelain
    #[arg(short, long, value_name = "FMT")]
    pub format: Option<String>,

    /// stale なワークスペースも表示
    #[arg(short, long)]
    pub all: bool,

    /// 現在のワークスペース情報のみ表示
    #[arg(short, long)]
    pub current: bool,
}

pub fn run(
    args: &ListArgs,
    config: &Config,
    printer: &Printer,
    repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    let mut workspaces = WorkspaceInfo::load_all(repo_root)?;

    if args.current {
        workspaces.retain(|w| w.is_current);
    } else if !args.all && !config.list.show_stale {
        workspaces.retain(|w| !w.is_stale);
    }

    let fmt = args
        .format
        .as_deref()
        .unwrap_or(&config.list.format)
        .to_owned();

    match fmt.as_str() {
        "json" => print_json(&workspaces),
        "porcelain" => print_porcelain(&workspaces),
        "simple" => print_simple(&workspaces, printer),
        _ => print_table(&workspaces, printer),
    }

    Ok(())
}

fn print_table(workspaces: &[WorkspaceInfo], printer: &Printer) {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_header(vec![
        Cell::new("DOMAIN").add_attribute(Attribute::Bold),
        Cell::new("PATH").add_attribute(Attribute::Bold),
        Cell::new("CHANGE").add_attribute(Attribute::Bold),
        Cell::new("DESCRIPTION").add_attribute(Attribute::Bold),
        Cell::new("STATUS").add_attribute(Attribute::Bold),
    ]);

    for ws in workspaces {
        let name_cell = if printer.color {
            if ws.is_current {
                Cell::new(ws.display_name(true))
                    .fg(Color::Cyan)
                    .add_attribute(Attribute::Bold)
            } else if ws.is_stale {
                Cell::new(ws.display_name(false)).fg(Color::Yellow)
            } else {
                Cell::new(ws.display_name(false))
            }
        } else {
            Cell::new(ws.display_name(true))
        };

        let path_cell = if printer.color {
            Cell::new(ws.display_path()).fg(Color::Blue)
        } else {
            Cell::new(ws.display_path())
        };

        let change_cell = {
            let text = ws.change_id.as_deref().unwrap_or("(stale)");
            if printer.color && !ws.is_stale {
                Cell::new(text).fg(Color::Magenta)
            } else if ws.is_stale && printer.color {
                Cell::new(text).fg(Color::Yellow)
            } else {
                Cell::new(text)
            }
        };

        let desc_cell = Cell::new(ws.description.as_deref().unwrap_or(""));

        let status_cell = if ws.is_stale && printer.color {
            Cell::new("stale").fg(Color::Yellow)
        } else {
            Cell::new(ws.status_str())
        };

        table.add_row(vec![name_cell, path_cell, change_cell, desc_cell, status_cell]);
    }

    println!("{table}");
}

fn print_simple(workspaces: &[WorkspaceInfo], printer: &Printer) {
    for ws in workspaces {
        let name = printer.domain_name(&ws.display_name(true), ws.is_current);
        let path = printer.path_text(&ws.display_path());
        println!("{name}\t{path}");
    }
}

fn print_json(workspaces: &[WorkspaceInfo]) {
    println!("{}", serde_json::to_string_pretty(workspaces).unwrap());
}

fn print_porcelain(workspaces: &[WorkspaceInfo]) {
    for ws in workspaces {
        let current_marker = if ws.is_current { "\tcurrent" } else { "" };
        println!("{}\t{}{}", ws.name, ws.path.display(), current_marker);
    }
}
