use std::io::{self, Write};
use zz_account::wallet::Wallet;

pub fn run_repl() {
    let mut wallet = Wallet::new(None, None);

    println!("zz-wallet repl");
    println!("commands: show | save | load | backup [path] | recovery [path] | exit");

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            continue;
        }

        let parts: Vec<_> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "show" => {
                println!("{}", wallet.address.to_string());
            }

            "save" => {
                match wallet.save() {
                    Ok(_) => println!("saved"),
                    Err(e) => tracing::error!("save failed: {}", e),
                }
            }

            "load" => {
                match wallet.load() {
                    Ok(_) => println!("{}", wallet.address.to_string()),
                    Err(e) => tracing::error!("load failed: {}", e),
                }
            }

            "backup" => {
                let path = parts.get(1).map(|s| *s);
                match wallet.backup(path) {
                    Ok(p) => println!("backup: {}", p),
                    Err(e) => tracing::error!("backup failed: {}", e),
                }
            }

            "recovery" => {
                let path = parts.get(1).map(|s| *s);
                match wallet.recovery(path) {
                    Ok(_) => println!("{}", wallet.address.to_string()),
                    Err(e) => tracing::error!("recovery failed: {}", e),
                }
            }

            "exit" | "quit" => {
                break;
            }

            _ => println!("unknown command"),
        }
    }
}
