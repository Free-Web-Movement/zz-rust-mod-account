use bip39::{ Language, Mnemonic };
use bitcoin::address::AddressType;
use bitcoin::bip32::{ DerivationPath, Xpriv };
use bitcoin::{ Address, Network, PrivateKey, PublicKey };
use hmac::Hmac;
use pbkdf2::pbkdf2;
use secp256k1::{ Message, Secp256k1, ecdsa::Signature };
use sha2::{ Digest, Sha256, Sha512 };
use std::{ fmt, fs };
use std::io::{ self, Write, Read };
use std::str::FromStr;

use serde::{ Serialize, Deserialize };

use crate::basic::{
    COIN_PREFIX,
    DERIVATION_PATH,
    MNEMONIC_SEED_ROUNDS,
    MNEMONIC_SEED_SIZE,
    MNEMONIC_STR,
    MNEMONIC_WORD_COUNT,
};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MnemonicInfo {
    pub language: Language,
    pub word_count: usize,
    pub phrase: String,
    pub passphrase: String,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AddressInfo {
    pub derivation_path: String,
    pub network: Network,
    pub address_type: AddressType,
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct FreeWebMovementAddress {
    #[serde(with = "crate::address::serde_prefix")]
    pub prefix: String,
    #[serde(with = "crate::address::serde_mnemonic")]
    mnemonic: Mnemonic,
    #[serde(with = "crate::address::serde_address")]
    address: Address,
    #[serde(with = "crate::address::serde_pubkey")]
    pub(crate) public_key: PublicKey,
    #[serde(with = "crate::address::serde_privkey")]
    pub(crate) private_key: PrivateKey,
}

pub mod serde_prefix {
    use super::*;
    use serde::{ Serializer, Deserializer };

    pub fn serialize<S>(prefix: &str, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(prefix)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(s)
    }
}
pub mod serde_mnemonic {
    use super::*;
    use serde::{ Serializer, Deserializer };

    pub fn serialize<S>(mnemonic: &Mnemonic, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&mnemonic.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mnemonic, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Mnemonic::parse(&s).map_err(serde::de::Error::custom)
    }
}

pub mod serde_address {
    use super::*;
    use serde::{ Serializer, Deserializer };

    pub fn serialize<S>(address: &Address, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&address.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Address, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let unchecked = Address::from_str(&s).map_err(serde::de::Error::custom)?;
        unchecked.require_network(Network::Bitcoin).map_err(serde::de::Error::custom)
    }
}

pub mod serde_pubkey {
    use super::*;
    use serde::{ Serializer, Deserializer };

    pub fn serialize<S>(pk: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&pk.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

pub mod serde_privkey {
    use super::*;
    use serde::{ Serializer, Deserializer };

    pub fn serialize<S>(sk: &PrivateKey, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&sk.to_wif())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PrivateKey, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        PrivateKey::from_wif(&s).map_err(serde::de::Error::custom)
    }
}

#[allow(dead_code)]
impl FreeWebMovementAddress {
    pub fn new(mnemonic_info: MnemonicInfo, address_info_option: Option<AddressInfo>) -> Self {
        let mnemonic: Mnemonic;
        if mnemonic_info.phrase.is_empty() {
            mnemonic = Mnemonic::generate_in(
                mnemonic_info.language,
                mnemonic_info.word_count
            ).unwrap();
        } else {
            mnemonic = Mnemonic::parse_in(
                mnemonic_info.language,
                mnemonic_info.phrase.clone()
            ).unwrap();
        }

        // 默认地址信息

        let address_info = address_info_option.unwrap_or(AddressInfo {
            derivation_path: DERIVATION_PATH.to_string(),
            network: Network::Bitcoin,
            address_type: AddressType::P2pkh,
            prefix: COIN_PREFIX.to_string(),
        });

        let seed: [u8; MNEMONIC_SEED_SIZE] = FreeWebMovementAddress::mnemonic_to_seed(
            &mnemonic.clone(),
            &mnemonic_info.passphrase.clone()
        );
        let (public_key, private_key) = FreeWebMovementAddress::to_key_pair(
            seed,
            &address_info.derivation_path,
            address_info.network
        ).unwrap();
        let address = FreeWebMovementAddress::key_to_inner_address(
            public_key,
            address_info.network,
            address_info.address_type
        ).unwrap();

        FreeWebMovementAddress {
            prefix: address_info.prefix,
            mnemonic,
            address,
            public_key,
            private_key,
        }
    }

    // Basic functions

    pub fn mnemonic_to_seed(mnemonic: &Mnemonic, passphrase: &str) -> [u8; MNEMONIC_SEED_SIZE] {
        let mut seed = [0u8; MNEMONIC_SEED_SIZE];
        let salt = format!("{}{}", MNEMONIC_STR, passphrase);
        let _ = pbkdf2::<Hmac<Sha512>>(
            mnemonic.to_string().as_bytes(),
            salt.as_bytes(),
            MNEMONIC_SEED_ROUNDS,
            &mut seed
        );
        seed
    }

    pub fn key_to_inner_address(
        key: PublicKey,
        network: Network,
        address_type: AddressType // 新增参数
    ) -> Result<Address, String> {
        let address = match address_type {
            AddressType::P2pkh => Address::p2pkh(&key, network),
            AddressType::P2wpkh => Address::p2wpkh(&key, network).unwrap(),
            AddressType::P2sh => Address::p2shwpkh(&key, network).unwrap(),
            _ => {
                return Err("Unsupported address type".to_string());
            }
        };
        Ok(address)
    }

    pub fn to_key_pair(
        seed: [u8; MNEMONIC_SEED_SIZE],
        dp: &str,
        network: Network
    ) -> Result<(PublicKey, PrivateKey), String> {
        let secp = Secp256k1::new();
        let xprv = Xpriv::new_master(network, &seed).map_err(|e| e.to_string())?;
        let path = DerivationPath::from_str(dp).map_err(|e| e.to_string())?;
        let child_prv = xprv.derive_priv(&secp, &path).map_err(|e| e.to_string())?;
        let child_pub = child_prv.to_priv().public_key(&secp);
        Ok((child_pub, child_prv.to_priv()))
    }

    pub fn sign_message(private_key: &PrivateKey, msg: &[u8]) -> Signature {
        let secp = Secp256k1::new();
        let secret = private_key.inner;
        let hash = Sha256::digest(msg);
        let message = Message::from_digest_slice(&hash).unwrap();
        secp.sign_ecdsa(&message, &secret)
    }

    pub fn verify_message(public_key: &PublicKey, msg: &[u8], signature: &Signature) -> bool {
        let secp = Secp256k1::new();
        let hash = Sha256::digest(msg);
        let message = Message::from_digest_slice(&hash).unwrap();
        secp.verify_ecdsa(&message, signature, &public_key.inner).is_ok()
    }

    pub fn random() -> Self {
        let mnemonic_info = MnemonicInfo {
            language: Language::English,
            word_count: MNEMONIC_WORD_COUNT,
            phrase: String::new(),
            passphrase: String::new(),
        };

        FreeWebMovementAddress::new(mnemonic_info, None)
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let json = serde_json
            ::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn from_json(json: &str) -> io::Result<Self> {
        let addr = serde_json::from_str(json).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(addr)
    }

    /// 从文件读取地址
    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let mut file = fs::File::open(path)?;
        let mut json = String::new();
        file.read_to_string(&mut json)?;
        FreeWebMovementAddress::from_json(&json)
    }
}

impl fmt::Display for FreeWebMovementAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.prefix, self.address)
    }
}

#[cfg(test)]
mod tests {
    use crate::address::AddressInfo;
    use crate::address::FreeWebMovementAddress;
    use crate::address::MnemonicInfo;
    // use crate::address::COIN_PREFIX;
    // use crate::address::DERIVATION_PATH;
    // use crate::address::MNEMONIC_WORD_COUNT;
    use bip39::Language;
    use bip39::Mnemonic;
    use bitcoin::AddressType;
    use bitcoin::Network;
    use crate::basic::*;
    #[test]
    fn it_should_generate() {
        let mi_en: MnemonicInfo = MnemonicInfo {
            language: Language::English,
            word_count: MNEMONIC_WORD_COUNT,
            phrase: String::new(),
            passphrase: String::new(),
        };

        let mut mi_scn: MnemonicInfo = mi_en.clone();
        let mut mi_tcn: MnemonicInfo = mi_en.clone();
        mi_scn.language = Language::SimplifiedChinese;
        mi_tcn.language = Language::TraditionalChinese;

        println!(
            "生成助记词: {}",
            Mnemonic::generate_in(mi_en.language, mi_en.word_count).unwrap().to_string()
        );
        println!(
            "生成助记词: {}",
            Mnemonic::generate_in(mi_scn.language, mi_en.word_count).unwrap().to_string()
        );
        println!(
            "生成助记词: {}",
            Mnemonic::generate_in(mi_tcn.language, mi_en.word_count).unwrap().to_string()
        );

        let ai = AddressInfo {
            derivation_path: DERIVATION_PATH.to_string(),
            network: Network::Bitcoin,
            address_type: AddressType::P2pkh, // 默认使用 P2pkh 地址类型
            prefix: COIN_PREFIX.to_string(),
        };

        let mut ai1 = ai.clone();
        let mut ai2 = ai.clone();
        let mut ai3 = ai.clone();
        let mut ai4 = ai.clone();
        ai1.address_type = AddressType::P2wpkh; // 设置为 P2wpkh 地址类型
        ai2.address_type = AddressType::P2sh;
        ai3.address_type = AddressType::P2tr;
        ai4.address_type = AddressType::P2pkh;

        let mi_en_phrase: MnemonicInfo = MnemonicInfo {
            language: Language::English,
            word_count: MNEMONIC_WORD_COUNT,
            phrase: String::from(
                "legal winner thank year wave sausage worth useful legal winner thank yellow"
            ),
            passphrase: String::new(),
        };

        let mi_en_phrase1: MnemonicInfo = MnemonicInfo {
            language: Language::English,
            word_count: MNEMONIC_WORD_COUNT,
            phrase: String::from(
                "legal winner thank year wave sausage worth useful legal winner thank yellow"
            ),
            passphrase: String::new(),
        };

        let mi_en_phrase2: MnemonicInfo = MnemonicInfo {
            language: Language::English,
            word_count: MNEMONIC_WORD_COUNT,
            phrase: String::from(
                "public refuse price sadness winter nose finger bomb damage corn expect marble"
            ),
            passphrase: String::new(),
        };

        let fwmaddress = FreeWebMovementAddress::new(mi_en.clone(), None);
        let fwmaddress1 = FreeWebMovementAddress::new(mi_en.clone(), None);
        let fwmaddress2 = FreeWebMovementAddress::new(mi_en.clone(), Some(ai1.clone()));
        let fwmaddress3 = FreeWebMovementAddress::new(mi_en_phrase, Some(ai2.clone()));
        let fwmaddress4 = FreeWebMovementAddress::new(mi_en_phrase1, Some(ai2.clone()));
        let fwmaddress5 = FreeWebMovementAddress::new(mi_en_phrase2, Some(ai2.clone()));
        // let fwmaddress6 = FreeWebMovementAddress::new(mi_en.clone(), Some(ai3.clone()));
        // assert!(fwmaddress6.is_err());

        assert_eq!(fwmaddress3.to_string(), fwmaddress4.to_string());
        assert_ne!(fwmaddress5.to_string(), fwmaddress4.to_string());

        println!("生成地址: {}", fwmaddress.to_string());
        println!("生成地址: {}", fwmaddress1.to_string());
        println!("生成地址: {}", fwmaddress2.to_string());
        println!("生成地址: {}", fwmaddress3.to_string());
        println!("生成地址: {}", FreeWebMovementAddress::random().to_string());

        let message = "Hello, FWM!".as_bytes();
        let signature = FreeWebMovementAddress::sign_message(&fwmaddress.private_key, message);
        let is_valid = FreeWebMovementAddress::verify_message(
            &fwmaddress.public_key,
            message,
            &signature
        );
        assert!(is_valid, "签名验证失败!");
    }

    #[test]
    fn test_fwmaddress_serde() {
        let fwmaddress = FreeWebMovementAddress::random();

        // 序列化为 JSON
        let json = serde_json::to_string(&fwmaddress).expect("序列化失败");
        println!("FreeWebMovementAddress JSON: {}", json);

        // 反序列化回对象
        let fwmaddress2: FreeWebMovementAddress = serde_json
            ::from_str(&json)
            .expect("反序列化失败");

        // 关键字段应一致
        assert_eq!(fwmaddress.to_string(), fwmaddress2.to_string());
        assert_eq!(fwmaddress.prefix, fwmaddress2.prefix);
        assert_eq!(fwmaddress.public_key, fwmaddress2.public_key);
        assert_eq!(fwmaddress.private_key, fwmaddress2.private_key);
        assert_eq!(fwmaddress.mnemonic.to_string(), fwmaddress2.mnemonic.to_string());
        assert_eq!(fwmaddress.address, fwmaddress2.address);
    }

    #[test]
    fn test_save_and_load_fwmaddress() {
        let fwmaddress = FreeWebMovementAddress::random();
        let path = "/tmp/fwmaddress.json";
        fwmaddress.save_to_file(path).expect("保存失败");
        let loaded = FreeWebMovementAddress::load_from_file(path).expect("读取失败");
        assert_eq!(fwmaddress.to_string(), loaded.to_string());
        assert_eq!(fwmaddress.public_key, loaded.public_key);
        assert_eq!(fwmaddress.private_key, loaded.private_key);
        assert_eq!(fwmaddress.mnemonic.to_string(), loaded.mnemonic.to_string());
        assert_eq!(fwmaddress.address, loaded.address);
        // 清理
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_basics() {
        println!("MAX_HUMAN_POPULATION: {}", MAX_HUMAN_POPULATION);
        println!("AVERAGE_ASSETS_PER_ONE_IN_USD: {}", AVERAGE_ASSETS_PER_USER_AS_IN_USD);
        println!("MAX_COIN_SUPPLY: {}", MAX_COIN_SUPPLY);
        println!("COIN_NAME: {}", COIN_NAME);
        println!("COIN_SYMBOL: {}", COIN_SYMBOL);
        println!("COIN_DECIMALS: {}", COIN_DECIMALS);
        assert_eq!(MAX_HUMAN_POPULATION, 10_000_000_000);
        assert_eq!(AVERAGE_ASSETS_PER_USER_AS_IN_USD, 1_000_000);
        assert_eq!(MAX_COIN_SUPPLY, MAX_HUMAN_POPULATION * AVERAGE_ASSETS_PER_USER_AS_IN_USD * 100); // 10 trillion USD
        assert_eq!(COIN_NAME, "Free Web Movement Coin - Zero Trust Zero Governance");
        assert_eq!(COIN_SYMBOL, "FWMC-ZZ");
        assert_eq!(COIN_DECIMALS, 8); // 1 ZZC = 0.00000001 USD
    }
}


