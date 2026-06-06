#!/bin/bash

cargo build --release
sudo cp target/release/zephyr_vault /usr/local/bin/
echo "Installation terminée. Tapez 'zephyr_vault --help'"