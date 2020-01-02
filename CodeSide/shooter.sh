set -ex

cargo build --release || { echo 'build failed' ; exit 1; }

cd $LOCAL_RUNNER_DIR
./aicup2019 --config flat_config.json &
sleep .5


cd $STRATEGY_DIR/target/release
./aicup2019 127.0.0.1 31001 token shooter &
sleep .5

./aicup2019 127.0.0.1 31000 token "@"
