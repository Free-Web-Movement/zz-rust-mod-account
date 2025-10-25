use jni::objects::{ JClass, JString };
use jni::sys::{ jlong };
use jni::JNIEnv;
use crate::address::FreeWebMovementAddress;

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
    let address = unsafe { &mut *(ptr as *mut FreeWebMovementAddress) };

    let prefix = env.new_string(&address.prefix).expect("Couldn't create Java string");
    prefix
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_toString<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = unsafe { &mut *(ptr as *mut FreeWebMovementAddress) };

    let str = env.new_string(&address.to_string()).expect("Couldn't create Java string");
    str
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_rs_zz_coin_Address_privateKey<'a>(
    env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JString<'a> {
    let address = unsafe { &mut *(ptr as *mut FreeWebMovementAddress) };

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
    let address = unsafe { &mut *(ptr as *mut FreeWebMovementAddress) };

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
    let address = unsafe { &mut *(ptr as *mut FreeWebMovementAddress) };
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
    let addr = FreeWebMovementAddress::from_json(&str).expect("Failed to deserialize FreeWebMovementAddress from JSON");
    Box::into_raw(Box::new(addr)) as jlong
}

#[test]
fn test_create_address() {
    use jni::InitArgsBuilder;
    use jni::JavaVM;
    use crate::basic::COIN_PREFIX;

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

    unsafe {
        let class = JClass::default();
        let env = jvm.attach_current_thread_permanently().unwrap();

        Java_rs_zz_coin_Address_destroy(env, class, address_ptr_2);
    }
}
