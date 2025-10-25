# RUSTFLAGS="-C instrument-coverage" cargo build
# RUSTFLAGS="-C instrument-coverage" cargo test --tests

ANDROID_NDK_HOME=/disk2vm/Android/Sdk/ndk/29.0.13599879
export ANDROID_NDK_HOME
export PATH=$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
export RUSTFLAGS="-Awarnings"
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o ./jniLibs build --release
# cargo ndk -p 21 -t aarch64-linux-android -t armv7-linux-androideabi -t i686-linux-android -t x86_64-linux-android -o ./jniLibs build --release
