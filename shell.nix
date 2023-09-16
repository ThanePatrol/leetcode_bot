let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "moz_overlay_shell";
    buildInputs = [
      (nixpkgs.latest.rustChannels.stable.rust.override {
          targets = [
            "x86_64-unknown-linux-gnu"
          ];
      })

      openssl
      pkg-config
      libsodium
      zlib
      binutils
      llvmPackages_rocm.lld
      lld_9
    ];


  }
