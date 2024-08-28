Command line tool for using MSRX6 device from command line.

## TODO

Some preliminary todo list

- [x] Detect if device is connected. Show error message
- [x] needs to read the data
- [x] needs to write data
- [x] input validation for the tracks (length and supported characters per track)
- [ ] ~~Allow only 1 or 2 tracks to be written~~ Seems that this can't be done with ISO writing
- [x] handle timeout when card not swiped (read & write)
- formatter
  - default:
    - read: use same format as basic readers, e.g. `` %QWERTYUIOPASDFGHJKLZXCVBNM_`01234567890123456789\_ ``
- [w] needs to work in RPI
  - [x] local build works when cloning the repo
  - [ ] at least make command that builds the bin
  - [ ] CI/CD pipeline setup?
- [ ] [OPTIONAL] Combine input and output formats to on "Format" enum and implement `to` and `from` functions

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
