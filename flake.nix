{
  description = "jacobhenn's Rust dev flake";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          name = "rust-dev";
          buildInputs = [
            pkg-config
            (
              rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ "x86_64-unknown-linux-gnu" ];
              })
            )
          ] ++ lib.optionals (lib.strings.hasInfix "linux" system) [
            # for Linux
            # Audio (Linux only)
            alsa-lib
            # Cross Platform 3D Graphics API
            vulkan-loader
            # For debugging around vulkan
            vulkan-tools
            # Other dependencies
            libudev-zero
            libx11
            libxcursor
            libxi
            libxrandr
            libxkbcommon
            wayland
            wayland.dev
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath [
              vulkan-loader
              libx11
              libxi
              libxcursor
              libxkbcommon
          ];
        };
      }
    );
}
