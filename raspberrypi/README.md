# VolP 送信側

## ハードウェア

- USB マイク
- RaspberryPi 4B (Ubuntu 22.10)
  - Zero 2 W でやりたかったが、さまざまな理由で完成しなかった
    - Raspberry Pi OS は OpenSSL まわりが面倒だった。.so が無かったがセルフビルドも頓挫
      - rustls を使うも、native-tls への依存を 0 にするのが難しかった
    - Ubuntu の使用を検討したが、Desktop 版は Zero 2 W じゃメモリ不足
    - だからといって Ubuntu Server だと今度は音声周り(ALSA)で問題が出た
  - Zero 2 W + 軽量 Linux で活路があるかも

## 必要なパッケージ

- gcc
- pkg-config
- libssl-dev
- libasound2-dev
- rpi.gpio-common

## 準備

sudo apt update  
sudo apt upgrade

gpio を root でなくても使えるようにする  
$ sudo adduser "\${USER}" dialout  
$ sudo reboot

マイクの音量を最大にする  
$ sudo alsamixer
