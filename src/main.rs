use clap::Parser;
use rmz::cli::Cli;
use rmz::commands::execute_command;

fn main() -> anyhow::Result<()> {
    // 環境変数やログ設定の初期化
    #[cfg(feature = "colors")]
    colored::control::set_override(true);

    let cli = Cli::parse();
    execute_command(cli)
}
