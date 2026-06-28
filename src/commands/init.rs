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
    args: &InitArgs,
    _config: &Config,
    _printer: &Printer,
) -> anyhow::Result<()> {
    match args.shell.as_str() {
        "bash" => emit_bash(args.hook_key.as_deref()),
        "zsh" => emit_zsh(args.hook_key.as_deref()),
        "fish" => emit_fish(args.hook_key.as_deref()),
        "nushell" | "nu" => emit_nushell(),
        other => anyhow::bail!(
            "unsupported shell \"{other}\". Supported: bash, zsh, fish, nushell"
        ),
    }

    Ok(())
}

// --------------------------------------------------------------------------
// Shell script templates stored as byte arrays to avoid format-string issues.
// --------------------------------------------------------------------------

const BASH_SCRIPT: &str = "\
# ryoiki shell integration — bash\n\
ryoiki() {\n\
  local _output _exit _cd_path\n\
  _output=\"$(command ryoiki --shell-output \"$@\" 2>&1)\"\n\
  _exit=$?\n\
  _cd_path=\"$(printf '%s\\n' \"$_output\" | grep '^RYOIKI:CD:' | tail -1 | cut -d: -f3-)\"\n\
  printf '%s\\n' \"$_output\" | grep -v '^RYOIKI:'\n\
  if [[ $_exit -ne 0 ]]; then\n\
    return $_exit\n\
  fi\n\
  if [[ -n \"$_cd_path\" ]]; then\n\
    builtin cd \"$_cd_path\"\n\
  fi\n\
}\n\
\n\
ryoiki_current_domain() {\n\
  command ryoiki list --format porcelain 2>/dev/null \\\n\
    | awk -F'\\t' '$3==\"current\"{print $1}'\n\
}\n\
";

const ZSH_SCRIPT: &str = "\
# ryoiki shell integration — zsh\n\
ryoiki() {\n\
  local _output _exit _cd_path\n\
  _output=\"$(command ryoiki --shell-output \"$@\" 2>&1)\"\n\
  _exit=$?\n\
  _cd_path=\"$(printf '%s\\n' \"$_output\" | grep '^RYOIKI:CD:' | tail -1 | cut -d: -f3-)\"\n\
  printf '%s\\n' \"$_output\" | grep -v '^RYOIKI:'\n\
  if [[ $_exit -ne 0 ]]; then\n\
    return $_exit\n\
  fi\n\
  if [[ -n \"$_cd_path\" ]]; then\n\
    builtin cd \"$_cd_path\"\n\
  fi\n\
}\n\
\n\
ryoiki_current_domain() {\n\
  command ryoiki list --format porcelain 2>/dev/null \\\n\
    | awk -F'\\t' '$3==\"current\"{print $1}'\n\
}\n\
";

const FISH_SCRIPT: &str = "\
# ryoiki shell integration — fish\n\
function ryoiki\n\
    set -l _output (command ryoiki --shell-output $argv 2>&1)\n\
    set -l _exit $status\n\
    set -l _cd_path (printf '%s\\n' $_output | grep '^RYOIKI:CD:' | tail -1 | string replace -r '^RYOIKI:CD:' '')\n\
    printf '%s\\n' $_output | grep -v '^RYOIKI:'\n\
    if test $_exit -ne 0\n\
        return $_exit\n\
    end\n\
    if test -n \"$_cd_path\"\n\
        builtin cd $_cd_path\n\
    end\n\
end\n\
\n\
function ryoiki_current_domain\n\
    command ryoiki list --format porcelain 2>/dev/null \\\n\
        | awk -F'\\t' '$3==\"current\"{print $1}'\n\
end\n\
";

const NUSHELL_SCRIPT: &str = "\
# ryoiki shell integration — nushell\n\
# Save this output and source it from config.nu:\n\
#   ryoiki init nushell | save ~/.config/ryoiki/integration.nu\n\
#   source ~/.config/ryoiki/integration.nu\n\
\n\
def-env ryoiki [...args] {\n\
    let output = (^ryoiki --shell-output ...$args | complete)\n\
    let lines = ($output.stdout | lines)\n\
    let cd_line = ($lines | where { |l| $l | str starts-with 'RYOIKI:CD:' } | last)\n\
    $lines | where { |l| not ($l | str starts-with 'RYOIKI:') } | str join \"\\n\" | print\n\
    if ($output.exit_code != 0) {\n\
        error make { msg: \"ryoiki exited with non-zero status\" }\n\
    }\n\
    if not ($cd_line | is-empty) {\n\
        let path = ($cd_line | str replace 'RYOIKI:CD:' '')\n\
        cd $path\n\
    }\n\
}\n\
\n\
def ryoiki_current_domain [] {\n\
    ^ryoiki list --format porcelain\n\
        | lines\n\
        | where { |l| $l | str contains \"\\tcurrent\" }\n\
        | first\n\
        | split column \"\\t\"\n\
        | get column1\n\
        | first\n\
}\n\
";

fn emit_bash(hook_key: Option<&str>) {
    use std::io::Write;
    std::io::stdout().write_all(BASH_SCRIPT.as_bytes()).unwrap();
    if let Some(key) = hook_key {
        println!("\n# Key binding: {key}");
        println!("bind -x '\"{}\"': \"ryoiki enter\"'", key);
    }
}

fn emit_zsh(hook_key: Option<&str>) {
    use std::io::Write;
    std::io::stdout().write_all(ZSH_SCRIPT.as_bytes()).unwrap();
    if let Some(key) = hook_key {
        println!("\n# Key binding: {key}");
        println!("_ryoiki_enter_widget() {{ ryoiki enter; zle reset-prompt }}");
        println!("zle -N _ryoiki_enter_widget");
        println!("bindkey '{}' _ryoiki_enter_widget", key);
    }
}

fn emit_fish(hook_key: Option<&str>) {
    use std::io::Write;
    std::io::stdout().write_all(FISH_SCRIPT.as_bytes()).unwrap();
    if let Some(key) = hook_key {
        println!("\n# Key binding: {key}");
        println!("bind {} 'ryoiki enter; commandline -f repaint'", key);
    }
}

fn emit_nushell() {
    use std::io::Write;
    std::io::stdout()
        .write_all(NUSHELL_SCRIPT.as_bytes())
        .unwrap();
}
