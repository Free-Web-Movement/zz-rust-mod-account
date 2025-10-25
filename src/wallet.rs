use std::{ io::{ Read, Write }, path::{ Path, PathBuf } };

use crate::address::FWMAddress;

const DEFAULT_WALLET_DIR: &str = "./free-web-coin/wallets";
const DEFAULT_WALLET_FILE: &str = "wallet.json";

struct Wallet {
    pub address: FWMAddress,
    pub directory: String,
    pub filename: String,
}

impl Wallet {
    pub fn new(directory: Option<&str>, filename: Option<&str>) -> Self {
        // 获取用户主目录
        let mut dir = dirs::home_dir().expect("无法获取用户主目录");

        match directory {
            Some(dir_str) => {
                if dir_str.starts_with('/') {
                    // 如果是绝对路径，则直接使用
                    dir = PathBuf::from(dir_str);
                } else {
                    // 如果是相对路径，则拼接到用户主目录
                    dir.push(dir_str);
                }
            }
            None => {
                // 如果没有提供目录，则使用默认目录
                dir.push(DEFAULT_WALLET_DIR);
            }
        }

        // 检查目录是否存在，不存在则创建
        let path = Path::new(&dir);
        if !path.exists() {
            std::fs::create_dir_all(path).expect("Failed to create wallet directory");
        }

        let filename = filename.unwrap_or(DEFAULT_WALLET_FILE).to_string();
        let mut wallet_file = dir.clone();
        wallet_file.push(&filename);

        let address = if wallet_file.exists() {
            // 文件存在则读取
            let mut file = std::fs::File::open(&wallet_file).expect("无法打开钱包文件");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("无法读取钱包文件");
            serde_json::from_str(&contents).expect("钱包文件内容无效")
        } else {
            // 文件不存在则新建
            let addr = FWMAddress::random();
            let json = serde_json::to_string_pretty(&addr).expect("序列化钱包失败");
            let mut file = std::fs::File::create(&wallet_file).expect("无法创建钱包文件");
            file.write_all(json.as_bytes()).expect("无法写入钱包文件");
            addr
        };

        Self {
            address,
            directory: dir.as_os_str().to_string_lossy().to_string(),
            filename,
        }
    }

    pub fn to_absolute_path(&self) -> String {
        let mut path = PathBuf::from(&self.directory);
        path.push(self.filename.clone());
        path.to_string_lossy().to_string()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut file = std::fs::File::create(format!("{}", self.to_absolute_path()))?;
        let json = serde_json::to_string(&self.address)?;
        file.write(json.as_bytes())?;
        Ok(())
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let mut file = std::fs::File::open(format!("{}", self.to_absolute_path()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.address = serde_json::from_str(&contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wallet_create_save_load() {
        // 使用临时目录和文件名，避免污染真实钱包
        let test_dir = "/tmp/test_vigcoin_wallet";
        let test_file = "test_wallet.json";

        // 清理旧文件
        let _ = fs::remove_dir_all(test_dir);

        // 创建钱包
        let wallet = Wallet::new(Some(test_dir), Some(test_file));
        let addr1 = wallet.address.to_string();

        // 保存钱包
        wallet.save().expect("保存钱包失败");

        // 加载钱包
        let mut loaded_wallet = Wallet::new(Some(test_dir), Some(test_file));
        loaded_wallet.load().expect("加载钱包失败");
        let addr2 = loaded_wallet.address.to_string();

        // 检查地址一致
        assert_eq!(addr1, addr2);

        // 清理
        let _ = fs::remove_dir_all(test_dir);

        let wallet1 = Wallet::new(None, None);
        let wallet2 = Wallet::new(Some(DEFAULT_WALLET_DIR), None);
    }
}
