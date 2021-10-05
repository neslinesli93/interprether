#!/usr/bin/env bash

if [ $PWD == */scripts ]; then
    cd ..
fi

cargo make web
