use std::fs;
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;
use jni::objects::{ JClass, JString };
use jni::sys::{ jlong };
use jni::JNIEnv;
use crate::address::FreeWebMovementAddress;

#[inline(always)]
fn get_address_mut(ptr: jlong) -> &'static mut FreeWebMovementAddress {
    // 将 jlong 指针安全转换为 Rust 可用的可变引用
    unsafe {
        &mut *(ptr as *mut FreeWebMovementAddress)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_create(_env: JNIEnv, _: JClass) -> jlong {
    let addr = FreeWebMovementAddress::random();
    Box::into_raw(Box::new(addr)) as jlong
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_destroy(_env: JNIEnv, _: JClass, ptr: jlong) {
    let _ = unsafe { Box::from_raw(ptr as *mut FreeWebMovementAddress) };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_prefix<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let prefix = env.new_string(&address.info.prefix).expect("Couldn't create Java string");
    prefix
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_toString<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let str = env.new_string(&address.to_string()).expect("Couldn't create Java string");
    str
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_privateKey<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let hex_string = hex::encode(&&address.private_key.to_bytes());

    let str = env.new_string(hex_string.to_string()).expect("Couldn't create Java string");
    str
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_publicKey<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let hex_string = hex::encode(&&address.public_key.to_bytes());

    let str = env.new_string(hex_string.to_string()).expect("Couldn't create Java string");
    str
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_toJSON<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = get_address_mut(ptr);
    let json = serde_json
        ::to_string_pretty(address)
        .expect("Failed to serialize FreeWebMovementAddress to JSON");

    let str = env.new_string(json.to_string()).expect("Couldn't create Java string");
    str
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_fromJSON<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    json: JString
) -> jlong {
    let str = env.get_string(&json).expect("Couldn't get string from JString");
    let str = str.to_str().expect("Couldn't convert JString to str");
    let addr = FreeWebMovementAddress::from_json(&str).expect(
        "Failed to deserialize FreeWebMovementAddress from JSON"
    );
    Box::into_raw(Box::new(addr)) as jlong
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_save<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    path: JString<'a>
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let binding = env.get_string(&path).expect("无法获取路径");
    let path_str = binding.to_str().unwrap();
    let pathbuf = if path_str.is_empty() {
        // 默认路径使用 app_data/zz_wallet.json
        let mut dir = dirs::data_dir().expect("无法获取 app_data");
        dir.push("zz_wallet.json");
        dir
    } else {
        PathBuf::from(path_str)
    };

    if let Some(parent) = pathbuf.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("无法创建保存目录");
        }
    }

    let json = serde_json::to_string_pretty(address).expect("序列化失败");
    let mut file = fs::File::create(&pathbuf).expect("无法创建钱包文件");
    file.write_all(json.as_bytes()).expect("写入失败");

    env.new_string(pathbuf.to_string_lossy().to_string()).expect("无法创建返回字符串")
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_load<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    path: JString<'a>
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let binding = env.get_string(&path).expect("无法获取路径");
    let path_str = binding.to_str().unwrap();
    let pathbuf = if path_str.is_empty() {
        // 默认路径使用 app_data/zz_wallet.json
        let mut dir = dirs::data_dir().expect("无法获取 app_data");
        dir.push("zz_wallet.json");
        dir
    } else {
        PathBuf::from(path_str)
    };

    let json = fs::read_to_string(&pathbuf).expect("读取文件失败");
    *address = serde_json::from_str(&json).expect("反序列化失败");

    env.new_string(pathbuf.to_string_lossy().to_string()).expect("无法创建返回字符串")
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_backup<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    path: JString<'a>
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let binding = env.get_string(&path).expect("无法获取路径");
    let path_str = binding.to_str().unwrap();
    let pathbuf = if path_str.is_empty() {
        // 默认路径 app_data/backup_YYYYMMDD_HHMMSS.json
        let mut dir = dirs::data_dir().expect("无法获取 app_data");
        let ts = Local::now().format("%Y%m%d_%H%M%S");
        dir.push(format!("backup_{}.json", ts));
        dir
    } else {
        PathBuf::from(path_str)
    };

    if let Some(parent) = pathbuf.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("无法创建备份目录");
        }
    }

    let json = serde_json::to_string_pretty(address).expect("序列化失败");
    let mut file = fs::File::create(&pathbuf).expect("无法创建备份文件");
    file.write_all(json.as_bytes()).expect("写入失败");

    env.new_string(pathbuf.to_string_lossy().to_string()).expect("无法创建返回字符串")
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_recovery<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    path: JString<'a>
) -> JString<'a> {
    let address = get_address_mut(ptr);

    let binding = env.get_string(&path).expect("无法获取路径");
    let path_str = binding.to_str().unwrap();
    let pathbuf = if path_str.is_empty() {
        // 默认恢复使用 app_data 下最新的 backup_*.json
        let mut dir = dirs::data_dir().expect("无法获取 app_data");
        dir.push(""); // data_dir 本身就是目录
        let mut backups: Vec<PathBuf> = fs
            ::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.file_name()?.to_string_lossy().starts_with("backup_") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        backups.sort();
        backups.pop().expect("没有找到备份文件")
    } else {
        PathBuf::from(path_str)
    };

    let json = fs::read_to_string(&pathbuf).expect("读取备份文件失败");
    *address = serde_json::from_str(&json).expect("反序列化失败");

    env.new_string(pathbuf.to_string_lossy().to_string()).expect("无法创建返回字符串")
}

#[test]
fn test_create_address() {
    use jni::InitArgsBuilder;
    use jni::JavaVM;
    use crate::consts::COIN_PREFIX;

    let jvm_args = InitArgsBuilder::new().build().unwrap();
    let jvm = JavaVM::new(jvm_args).unwrap();
    let address_ptr = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        Java_rs_zz_coin_Address_create(env, class)
    };

    assert!(address_ptr != 0);

    let prefix = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        let str = Java_rs_zz_coin_Address_prefix(env, class, address_ptr);
        let mut env = jvm.attach_current_thread_permanently().unwrap();

        let prefix = env.get_string(&str).expect("Couldn't get string");
        prefix.to_str().unwrap().to_string()
    };

    assert_eq!(prefix, COIN_PREFIX.to_string());

    let address = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        let str = Java_rs_zz_coin_Address_toString(env, class, address_ptr);
        let mut env = jvm.attach_current_thread_permanently().unwrap();

        let local_str = env.get_string(&str).expect("Couldn't get string");
        local_str.to_str().unwrap().to_string()
    };

    println!("JNI Address: {}", address);

    assert!(address.contains(COIN_PREFIX));

    unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        Java_rs_zz_coin_Address_destroy(env, class, address_ptr);
    }

    let address_ptr = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        Java_rs_zz_coin_Address_create(env, class)
    };

    assert!(address_ptr != 0);
    let address_json = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        let json_str = Java_rs_zz_coin_Address_toJSON(env, class, address_ptr);
        let mut env = jvm.attach_current_thread_permanently().unwrap();

        let local_str = env.get_string(&json_str).expect("Couldn't get string");
        local_str.to_str().unwrap().to_string()
    };

    unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        Java_rs_zz_coin_Address_destroy(env, class, address_ptr);
    }

    let address_ptr_2 = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();
        let java_string: JString = env
            .new_string(&address_json)
            .expect("Couldn't create Java string!");

        Java_rs_zz_coin_Address_fromJSON(env, class, java_string)
    };

    assert!(address_ptr_2 != 0);

    let private_key_str = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        let str = Java_rs_zz_coin_Address_privateKey(env, class, address_ptr_2);
        let mut env = jvm.attach_current_thread_permanently().unwrap();

        let local_str = env.get_string(&str).expect("Couldn't get string");
        local_str.to_str().unwrap().to_string()
    };

    println!("Private Key: {}", private_key_str);

    let public_key_str = unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        let str = Java_rs_zz_coin_Address_publicKey(env, class, address_ptr_2);
        let mut env = jvm.attach_current_thread_permanently().unwrap();

        let local_str = env.get_string(&str).expect("Couldn't get string");
        local_str.to_str().unwrap().to_string()
    };

    println!("Public Key: {}", public_key_str);

    // 保存到临时文件
    let tmp_dir = std::env::temp_dir().join("jni_wallet_test");
    fs::create_dir_all(&tmp_dir).unwrap();
    let save_path = tmp_dir.join("wallet.json");
    let save_path_str = save_path.to_string_lossy().to_string();

    println!("save to {}", save_path_str);

    let jstr = unsafe {
        let class = JClass::default();

        let env = jvm.attach_current_thread_permanently().unwrap();
        let java_path = env.new_string(&save_path_str).unwrap();
        Java_rs_zz_coin_Address_save(env, class, address_ptr, java_path)
    };

    let saved_path = {
        let mut env = jvm.attach_current_thread_permanently().unwrap();
        env.get_string(&jstr).unwrap().to_str().unwrap().to_string()
    };
    assert!(PathBuf::from(&saved_path).exists());

    // load 测试
    unsafe {
        let class = JClass::default();

        let env = jvm.attach_current_thread_permanently().unwrap();
        let java_path = env.new_string(&save_path_str).unwrap();
        Java_rs_zz_coin_Address_load(env, class, address_ptr, java_path);
    }

    // backup 测试（使用默认 app_data）
    let backup_path = unsafe {
        let class = JClass::default();

        let env = jvm.attach_current_thread_permanently().unwrap();
        let java_path = env.new_string("").unwrap(); // 空字符串触发默认路径
        let jstr = Java_rs_zz_coin_Address_backup(env, class, address_ptr, java_path);

        let mut env = jvm.attach_current_thread_permanently().unwrap();
        env.get_string(&jstr).unwrap().to_str().unwrap().to_string()
    };
    assert!(PathBuf::from(&backup_path).exists());
    println!("Backup file: {}", backup_path);

        // recovery 测试
        let recovered_path = unsafe {
            let class = JClass::default();

            let env = jvm.attach_current_thread_permanently().unwrap();
            let java_path = env.new_string(&backup_path).unwrap();
            let jstr = Java_rs_zz_coin_Address_recovery(env, class, address_ptr, java_path);

            let mut env = jvm.attach_current_thread_permanently().unwrap();
            env.get_string(&jstr).unwrap().to_str().unwrap().to_string()
        };
        assert_eq!(recovered_path, backup_path);

        // 清理
        let _ = fs::remove_dir_all(&tmp_dir);
        let _ = fs::remove_file(&backup_path);

    unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        Java_rs_zz_coin_Address_destroy(env, class, address_ptr_2);
    }
}
