set -ex

cargo build --release
cd target/release
./aicup2019 127.0.0.1 31000 token "$@"
