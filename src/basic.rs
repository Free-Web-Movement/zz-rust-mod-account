pub const MAX_HUMAN_POPULATION: u64 = 10_000_000_000;
pub const AVERAGE_ASSETS_PER_USER_AS_IN_USD: u64 = 1_000_000;
pub const MAX_COIN_SUPPLY: u64 = MAX_HUMAN_POPULATION * AVERAGE_ASSETS_PER_USER_AS_IN_USD * 100; // 10 trillion USD
pub const COIN_NAME: &str = "Free Web Movement Coin";
pub const COIN_SYMBOL: &str = "FreeWebMovementCoin";
pub const COIN_DECIMALS: u8 = 8; // 1 ZZC = 0.00000001 USD

pub const DERIVATION_PATH: &str = "m/44'/1010086'/0'/0/0"; // 默认的派生路径
pub const MNEMONIC_STR: &str = "mnemonic"; // 默认的Mnemonic字符串前缀
pub const MNEMONIC_WORD_COUNT: usize = 24; // 默认的Word数量
pub const MNEMONIC_SEED_SIZE: usize = 64; // 默认的种子大小
pub const MNEMONIC_SEED_ROUNDS: u32 = 2048; // 默认的种子的轮数

pub const COIN_PREFIX: &str = "FreeWebMovementCoin:ZeroTrustZeroGovernance"; // 加密币前缀: FWM for Free Web Movement, Zz for Zero Trust, Zero Governance
