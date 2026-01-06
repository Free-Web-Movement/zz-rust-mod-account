use std::{ io::{ Read, Write }, path::{ Path, PathBuf } };

use crate::{ address::FreeWebMovementAddress, consts::{ DEFAULT_WALLET_DIR, DEFAULT_WALLET_FILE } };

pub struct Wallet {
    pub address: FreeWebMovementAddress,
    pub directory: String,
    pub filename: String,
}

impl Wallet {
    pub fn new(directory: Option<&str>, filename: Option<&str>) -> Self {
        // 获取用户 AppData 目录（Windows / macOS / Linux）
        let mut dir = dirs::data_dir().expect("无法获取用户 AppData 目录");

        match directory {
            Some(dir_str) => {
                if dir_str.starts_with('/') {
                    // 如果是绝对路径，则直接使用
                    dir = PathBuf::from(dir_str);
                } else {
                    // 如果是相对路径，则拼接到 AppData 目录
                    dir.push(dir_str);
                }
            }
            None => {
                // 如果没有提供目录，则使用默认目录
                dir.push(DEFAULT_WALLET_DIR);
            }
        }

        // 检查目录是否存在，不存在则创建
        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect("无法创建钱包目录");
        }

        let filename = filename.unwrap_or(DEFAULT_WALLET_FILE).to_string();
        let mut wallet_file = dir.clone();
        wallet_file.push(&filename);

        // 确保文件的父目录存在
        if let Some(parent) = wallet_file.parent() {
            std::fs::create_dir_all(parent).expect("无法创建钱包文件父目录");
        }

        // 如果文件存在则读取，否则新建随机地址
        let address = if wallet_file.exists() {
            let mut file = std::fs::File::open(&wallet_file).expect("无法打开钱包文件");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("无法读取钱包文件");
            serde_json::from_str(&contents).expect("钱包文件内容无效")
        } else {
            let addr = FreeWebMovementAddress::random();
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
   /// backup: 在指定路径生成 JSON，只保存 address
    /// 如果 path 为 None，则在 Wallet 的默认目录生成带时间戳的文件
    pub fn backup(&self, path: Option<&str>) -> std::io::Result<String> {
        // 生成 JSON 字符串
        let json = serde_json::to_string_pretty(&self.address)?;

        // 生成文件路径
        let backup_path = match path {
            Some(p) => {
                let mut pb = PathBuf::from(p);
                // 如果 p 是目录，则在目录下生成带时间戳的文件
                if pb.is_dir() || p.ends_with('/') {
                    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
                    pb.push(format!("wallet_backup_{}.json", timestamp));
                } else if !pb.exists() {
                    if let Some(parent) = pb.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                pb
            }
            None => {
                // 默认使用 Wallet 的目录
                let mut pb = PathBuf::from(&self.directory);
                let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
                pb.push(format!("wallet_backup_{}.json", timestamp));
                pb
            }
        };

        // 确保父目录存在
        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 写入文件
        let mut file = std::fs::File::create(&backup_path)?;
        file.write_all(json.as_bytes())?;

        Ok(backup_path.to_string_lossy().to_string())
    }

    /// recovery: 从指定 backup 文件恢复 address
    /// 如果 path 为 None，则从 Wallet 默认目录的最新备份文件恢复
    pub fn recovery(&mut self, path: Option<&str>) -> std::io::Result<()> {
        let backup_path = match path {
            Some(p) => PathBuf::from(p),
            None => {
                // 默认目录，选择最新备份文件
                let dir = PathBuf::from(&self.directory);
                let mut entries: Vec<_> = std::fs::read_dir(&dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                            && e.file_name().to_string_lossy().starts_with("wallet_backup_")
                            && e.file_name().to_string_lossy().ends_with(".json")
                    })
                    .collect();
                // 按文件名排序（时间戳靠前），取最后一个
                entries.sort_by_key(|e| e.file_name());
                if let Some(entry) = entries.pop() {
                    entry.path()
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "没有找到备份文件",
                    ));
                }
            }
        };

        // 读取文件
        let mut file = std::fs::File::open(&backup_path)?;
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

    
    #[test]
    fn test_wallet_create_save_load_backup_recovery_with_paths() {
        // 临时目录，避免污染真实钱包
        let tmp_dir = "/tmp/test_wallet_full";
        let wallet_file = "wallet.json";

        // 清理旧目录
        let _ = fs::remove_dir_all(tmp_dir);

        // 1. 创建钱包
        let wallet = Wallet::new(Some(tmp_dir), Some(wallet_file));
        let original_address = wallet.address.to_string();

        // 2. 保存钱包
        wallet.save().expect("保存钱包失败");

        // 3. 加载钱包
        let mut loaded_wallet = Wallet::new(Some(tmp_dir), Some(wallet_file));
        loaded_wallet.load().expect("加载钱包失败");
        assert_eq!(original_address, loaded_wallet.address.to_string());

        // 4. backup 指定绝对路径
        let backup_path_abs = format!("{}/backup_abs.json", tmp_dir);
        let backup_file_abs = loaded_wallet
            .backup(Some(&backup_path_abs))
            .expect("备份失败");
        assert!(PathBuf::from(&backup_file_abs).exists());

        // 5. backup 默认路径
        let backup_file_default = loaded_wallet
            .backup(None)
            .expect("默认路径备份失败");
        assert!(PathBuf::from(&backup_file_default).exists());

        // 6. recovery 指定绝对路径
        let mut wallet_recovered_abs = Wallet::new(Some(tmp_dir), Some(wallet_file));
        wallet_recovered_abs
            .recovery(Some(&backup_path_abs))
            .expect("指定路径恢复失败");
        assert_eq!(
            wallet_recovered_abs.address.to_string(),
            loaded_wallet.address.to_string()
        );

        // 7. recovery 默认路径（使用最后生成的备份文件）
        let mut wallet_recovered_default = Wallet::new(Some(tmp_dir), Some(wallet_file));
        wallet_recovered_default
            .recovery(None)
            .expect("默认路径恢复失败");
        assert_eq!(
            wallet_recovered_default.address.to_string(),
            loaded_wallet.address.to_string()
        );

        // 清理临时目录
        let _ = fs::remove_dir_all(tmp_dir);
    }
}
