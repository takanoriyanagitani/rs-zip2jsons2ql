#!/bin/sh

export ZIP_DIR="./zips.d"
export ITEM_SIZE_LIMIT="1048576"
export LISTEN_ADDR=127.0.0.1:8039

./zip2jsons2ql
