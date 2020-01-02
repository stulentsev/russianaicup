set -ex

cargo build --release || { echo 'build failed' ; exit 1; }

cd $LOCAL_RUNNER_DIR
./aicup2019 --config $CONFIG_FILE --save-results quick.json >/dev/null 2>&1 &
sleep .5


cd $STRATEGY_DIR/target/release
./aicup2019 127.0.0.1 31000 token "@"

cat $LOCAL_RUNNER_DIR/quick.json
