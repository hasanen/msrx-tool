Command line tool for using MSRX6 device from command line.

## TODO

- [ ] Rewrite

## Compile for RPI

Add target with rustup

    rustup target add aarch64-unknown-linux-gnu

Install arm-linux-gnueabihf-binutils

    brew install arm-linux-gnueabihf-binutils # on macOs
    sudo apt-get install gcc-aarch64-linux-gnu # on Debian based linux

## Setup

### Linux

Create udev rule `/etc/udev/rules.d/99-usb-msrx6.rules`

```bash
SUBSYSTEM=="usb", ATTR{idVendor}=="0801", ATTR{idProduct}=="0003", MODE="0666", GROUP="plugdev"
```

and reload rules

```bash
sudo udevadm control --reload
sudo udevadm trigger
```
