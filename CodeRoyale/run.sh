#! /bin/bash

./localrunner/aicup22 --config localrunner-configs/tcp-vs-quick.json --start-paused --antialias true --save-results results.json
cat results.json | jq -C '.results.players'