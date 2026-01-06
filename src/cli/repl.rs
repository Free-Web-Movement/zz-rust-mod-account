use std::io::{ self, Write };
use zz_account::wallet::Wallet;

pub fn run_repl() {
    let mut wallet = Wallet::new(None, None);

    println!("zz-wallet repl");
    println!("commands: show | save | load | backup [path] | recovery [path] | exit");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

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
                wallet.save().expect("save failed");
                println!("saved");
            }

            "load" => {
                wallet.load().expect("load failed");
                println!("{}", wallet.address.to_string());
            }

            "backup" => {
                let path = parts.get(1).map(|s| *s);
                let p = wallet.backup(path).expect("backup failed");
                println!("backup: {}", p);
            }

            "recovery" => {
                let path = parts.get(1).map(|s| *s);
                wallet.recovery(path).expect("recovery failed");
                println!("{}", wallet.address.to_string());
            }

            "exit" | "quit" => {
                break;
            }

            _ => println!("unknown command"),
        }
    }
}
