set -ex

if [ "$1" != "base" ]; then
    if [[ `ls -1 /src/ | wc -l` -eq 1 ]]; then
        cp -f /src/main_strategy src/main_strategy
    else
        rm -rf ./*
        cp -rf /src/* ./
    fi
fi

cargo build --release --offline
cp target/release/aicup2020 /output/
