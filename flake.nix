{
  description = "UEFI Memory Eraser";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    crane,
    fenix,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = (import nixpkgs) {inherit system;};
      toolchain = with fenix.packages.${system};
        combine [
          default.rustc
          default.cargo
          default.rustfmt
          targets.x86_64-unknown-uefi.latest.rust-std
        ];
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      src = craneLib.cleanCargoSource ./.;
    in rec {
      checks = {
        elusive-fmt = craneLib.cargoFmt {
          inherit src;
        };
      };

      formatter = pkgs.alejandra;

      apps.default = utils.lib.mkApp {
        drv = pkgs.writeShellScriptBin "elusive-qemu" ''
          mkdir -p '/tmp/elusive-qemu-esp/efi/boot'
          cp ${packages.default}/bin/elusive.efi /tmp/elusive-qemu-esp/efi/boot/bootx64.efi
          ${pkgs.qemu}/bin/qemu-system-x86_64 -enable-kvm \
            -drive if=pflash,format=raw,readonly=on,file=${pkgs.OVMF.fd}/FV/OVMF_CODE.fd \
            -drive if=pflash,format=raw,readonly=on,file=${pkgs.OVMF.fd}/FV/OVMF_VARS.fd \
            -drive format=raw,file=fat:rw:/tmp/elusive-qemu-esp
          rm -rf /tmp/elusive-qemu-esp
        '';
      };

      packages.default = craneLib.buildPackage {
        inherit src;

        strictDeps = true;
        doCheck = false;

        CARGO_BUILD_TARGET = "x86_64-unknown-uefi";
      };

      devShells.default = pkgs.mkShell {packages = [toolchain];};
    });
}
