use clap::Args;

use crate::{config::Config, output::Printer};

#[derive(Debug, Args)]
pub struct InitArgs {
    /// シェルの種類: bash / zsh / fish / nushell
    pub shell: String,

    /// enter コマンドにバインドするキー (例: \C-g)
    #[arg(long, value_name = "KEY")]
    pub hook_key: Option<String>,
}

pub fn run(
    _args: &InitArgs,
    _config: &Config,
    _printer: &Printer,
) -> anyhow::Result<()> {
    todo!("init command not yet implemented")
}
