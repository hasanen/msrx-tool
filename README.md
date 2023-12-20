Command line tool for using MSRX6 device from command line.

## TODO

Some preliminary todo list

- [x] Detect if device is connected. Show error message
- [ ] needs to read the data
  - [ ] `msrx-tool read` : reads all tracks
  - [ ] `msrx-tool read track[1-3]` : reads single track
  - [ ] save content to file? JSON?
- [ ] needs to write data
  - [ ] `msrx-tool write [track1_content] [track2_content] [track3_content]` : writes all tracks
  - [ ] `msrx-tool write track[1-3] [content]` : writes single track
  - [ ] read content to be written from file? JSON?
- formatter
  - default:
    - read: use same format as basic readers, e.g. `` %QWERTYUIOPASDFGHJKLZXCVBNM_`01234567890123456789\_ ``
- [ ] needs to work in RPI
  - [ ] at least make command that builds the bin
  - [ ] CI/CD pipeline setup?

## Compile for RPI

Add target with rustup

    rustup target add aarch64-unknown-linux-gnu

Install arm-linux-gnueabihf-binutils

    brew install arm-linux-gnueabihf-binutils # on macOs
    sudo apt-get install gcc-aarch64-linux-gnu # on Debian based linux
