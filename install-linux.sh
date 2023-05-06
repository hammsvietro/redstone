#!/bin/sh

cargo build --release
sudo su -c '
install -Dm 755 target/release/redstone /usr/bin/
install -Dm 755 target/release/redstone-service /usr/bin/
'
install -Dm 755 release/linux/redstone.service ~/.config/systemd/user/redstone.service
