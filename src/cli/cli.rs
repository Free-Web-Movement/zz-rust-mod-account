use clap::{Parser, Subcommand};
use zz_account::wallet::Wallet;

#[derive(Parser)]
#[command(name = "zz-wallet")]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    New {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Show {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Save {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Load {
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Backup {
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Recovery {
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Repl,
}

pub fn run_cli(cli: Cli) {
    match cli.command {
        Commands::New { dir, file } => {
            let wallet = Wallet::new(dir.as_deref(), file.as_deref());
            println!("{}", wallet.address.to_string());
        }

        Commands::Show { dir, file } => {
            let wallet = Wallet::new(dir.as_deref(), file.as_deref());
            println!("{}", wallet.address.to_string());
        }

        Commands::Save { dir, file } => {
            let wallet = Wallet::new(dir.as_deref(), file.as_deref());
            match wallet.save() {
                Ok(_) => println!("saved: {}", wallet.to_absolute_path()),
                Err(e) => tracing::error!("save failed: {}", e),
            }
        }

        Commands::Load { dir, file } => {
            let mut wallet = Wallet::new(dir.as_deref(), file.as_deref());
            match wallet.load() {
                Ok(_) => println!("{}", wallet.address.to_string()),
                Err(e) => tracing::error!("load failed: {}", e),
            }
        }

        Commands::Backup { path, dir, file } => {
            let wallet = Wallet::new(dir.as_deref(), file.as_deref());
            match wallet.backup(path.as_deref()) {
                Ok(backup_path) => println!("backup: {}", backup_path),
                Err(e) => tracing::error!("backup failed: {}", e),
            }
        }

        Commands::Recovery { path, dir, file } => {
            let mut wallet = Wallet::new(dir.as_deref(), file.as_deref());
            match wallet.recovery(path.as_deref()) {
                Ok(_) => println!("{}", wallet.address.to_string()),
                Err(e) => tracing::error!("recovery failed: {}", e),
            }
        }

        Commands::Repl => {
            crate::repl::run_repl();
        }
    }
}
