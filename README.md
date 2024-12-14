# Helious-rs
Modular hid-api-rs interface with [Battlebit anti-recoil](https://github.com/StrateimTech/helious-rs/blob/master/src/modules/recoil.rs) & remote state output

- Supports UART communication protocol
- External mouse state sending

## Installation
- Must have RPi 4b or newer (Must have USB OTG)
```
git clone https://github.com/StrateimTech/helious-rs
cd ./Helious
```
Architectures
- 64bit Armv8 - ``aarch64-unknown-linux-gnu``
- 32bit Armv7 - ``armv7-unknown-linux-gnueabihf``
```
cargo build --release --target=aarch64-unknown-linux-gnu
```
- First follow [hid-api-rs Gadget installation guide](https://github.com/StrateimTech/hid-api-rs?tab=readme-ov-file#first-installation). After verify if everything is setup correctly, ``ls`` into ``/dev/`` and see if ``hidg0`` and ``hidg1`` are present.
- Transfer ``helious-rs`` file within the target output directory to the RPi via your choice of method.
- Chmod & run with elevated permissions
```
chmod +x ./Helious
sudo ./Helious
```
![helious-console](https://github.com/user-attachments/assets/087b5587-6331-4400-823f-cbe82f84616b)

## BattleBit anti-recoil downsides
- Doesn't support horizontal recoil compensation. (Impossible)
- Doesn't support burst & single fire modes*

## Required Inputs
Stock AK74 for example:
- 1.40 = Vertical recoil
- 1.0 = First shot kick
- 670 = Fire rate
- 30 = Magazine capacity
- 110 = FOV

## Optional Inputs
- Global overflow: Compensates for loss in accuracy after each bullet.
- Local overflow: compensates for accuracy loss during smoothing process within each bullet.
