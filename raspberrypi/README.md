# VolP 送信側

## ハードウェア

- RaspberryPi Zero 2 W
- USB マイク

## 必要なパッケージ

- gcc
- pkg-config
- libssl-dev
- libasound2-dev
- rpi.gpio-common

## 準備
sudo apt update
sudo apt upgrade

gpioをrootでなくても使えるようにするために必要
$ sudo adduser "${USER}" dialout
$ sudo reboot

マイクの音量を最大にする
$ sudo alsamixer
