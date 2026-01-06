use clap::{ Parser, Subcommand };
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
            wallet.save().expect("save failed");
            println!("saved: {}", wallet.to_absolute_path());
        }

        Commands::Load { dir, file } => {
            let mut wallet = Wallet::new(dir.as_deref(), file.as_deref());
            wallet.load().expect("load failed");
            println!("{}", wallet.address.to_string());
        }

        Commands::Backup { path, dir, file } => {
            let wallet = Wallet::new(dir.as_deref(), file.as_deref());
            let backup_path = wallet.backup(path.as_deref()).expect("backup failed");
            println!("backup: {}", backup_path);
        }

        Commands::Recovery { path, dir, file } => {
            let mut wallet = Wallet::new(dir.as_deref(), file.as_deref());
            wallet.recovery(path.as_deref()).expect("recovery failed");
            println!("{}", wallet.address.to_string());
        }

        Commands::Repl => {
            crate::repl::run_repl();
        }
    }
}
