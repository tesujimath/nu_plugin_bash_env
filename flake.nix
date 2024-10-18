{
  description = "nu_plugin_bash_env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";

    flake-utils.url = "github:numtide/flake-utils";

    bash-env-json = {
      url = "github:tesujimath/bash-env-json?ref=refs/tags/0.6.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, bash-env-json, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];

          pkgs = import nixpkgs {
            inherit system overlays;
          };

          inherit (pkgs) rust-bin writeShellScriptBin;
          flakePkgs = {
            bash-env-json = bash-env-json.packages.${system}.default;
          };

          # cargo-nightly based on https://github.com/oxalica/rust-overlay/issues/82
          nightly = rust-bin.selectLatestNightlyWith (t: t.default);
          cargo-nightly = writeShellScriptBin "cargo-nightly" ''
            export RUSTC="${nightly}/bin/rustc";
            exec "${nightly}/bin/cargo" "$@"
          '';

          nu_plugin_bash_env =
            let
              inherit (pkgs) lib makeWrapper rustPlatform;
              cargoConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            in
            rustPlatform.buildRustPackage
              {
                pname = "nu_plugin_bash_env";
                version = cargoConfig.package.version;

                src = ./.;

                cargoLock = {
                  lockFile = ./Cargo.lock;
                };

                meta = with lib; {
                  description = "A Bash environment plugin for Nushell";
                  homepage = "https://github.com/tesujimath/nu_plugin_bash_env";
                  license = licenses.mit;
                  maintainers = [ maintainers.tailhook ];
                };

                buildInputs = [
                  makeWrapper
                  flakePkgs.bash-env-json
                ];


                preBuild = ''
                  export NIX_BASH_ENV_JSON=${flakePkgs.bash-env-json}/bin/bash-env-json
                '';

                postFixup = ''
                  wrapProgram $out/bin/nu_plugin_bash_env --set NU_PLUGIN_BASH_ENV_JSON ${flakePkgs.bash-env-json}/bin/bash-env-json
                '';
              };
        in
        {
          devShells.default =
            let
              inherit (pkgs)
                mkShell
                bashInteractive
                jq
                cargo-modules
                cargo-udeps
                rust-bin;
            in

            mkShell {
              nativeBuildInputs = [
                bashInteractive
                jq
                cargo-modules
                cargo-nightly
                cargo-udeps
                rust-bin.stable.latest.default
              ];
            };

          packages.default = nu_plugin_bash_env;
        }
      );
}
