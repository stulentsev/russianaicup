set -ex

cargo build --release || { echo 'build failed' ; exit 1; }

: > quick.json
cd $LOCAL_RUNNER_DIR
./aicup2020 --config $CONFIG_FILE --save-results quick.json >/dev/null 2>&1 &
sleep .5


cd $STRATEGY_DIR
./bin/aicup2020-v11 127.0.0.1 31001 token "$@" >/dev/null 2>&1 &
./bin/aicup2020-current 127.0.0.1 31000 token "$@"

cat $LOCAL_RUNNER_DIR/quick.json
