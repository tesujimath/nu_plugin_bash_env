{
  description = "nu_plugin_bash_env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          # cargo-nightly based on https://github.com/oxalica/rust-overlay/issues/82
          nightly = pkgs.rust-bin.selectLatestNightlyWith (t: t.default);
          cargo-nightly = pkgs.writeShellScriptBin "cargo-nightly" ''
            export RUSTC="${nightly}/bin/rustc";
            exec "${nightly}/bin/cargo" "$@"
          '';
          # TODO:
          # nu_plugin_bash_env = pkgs.writeShellScriptBin "nu_plugin_bash_env"
          #   (builtins.replaceStrings [ "jq" "(cat)" " sed " ] [ "${pkgs.jq}/bin/jq" "(${pkgs.coreutils}/bin/cat)" " ${pkgs.gnused}/bin/sed " ]
          #     (builtins.readFile ./nu_plugin_bash_env));
        in
        with pkgs;
        {
          devShells.default = mkShell {
            nativeBuildInputs = [
              bashInteractive
              jq
              cargo-modules
              cargo-nightly
              cargo-udeps
              rust-bin.stable.latest.default
            ];
          };
          # TODO: build package from Rust crate
          # packages.default = nu_plugin_bash_env;
        }
      );
}
