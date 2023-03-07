UEFI random-access memory eraser to protect against cold boot attacks.

## Build ##

```bash
sh <(curl -L https://nixos.org/nix/install) --daemon
mkdir -p ~/.config/nix
echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf
nix build
```

## Run in QEMU ##

```bash
nix run
```

## Install ##

```bash
nix build
cp result/bin/elusive.efi /boot/efi/
efibootmgr -c -L Elusive -d /dev/nvme0n1p1 -l '\efi\elusive.efi'
```

