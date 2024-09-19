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

          nu_plugin_bash_env_script = pkgs.writeShellScriptBin "nu_plugin_bash_env_script"
            (builtins.replaceStrings [ "jq" "(cat)" " sed " ] [ "${pkgs.jq}/bin/jq" "(${pkgs.coreutils}/bin/cat)" " ${pkgs.gnused}/bin/sed " ]
              (builtins.readFile ./scripts/bash_env.sh));

          nu_plugin_bash_env = pkgs.rustPlatform.buildRustPackage
            rec {
              pname = "nu_plugin_bash_env";
              version = "0.14.1";

              src = pkgs.fetchFromGitHub {
                owner = "tesujimath";
                repo = pname;
                rev = version;
                sha256 = "sha256-tXpO6W52MXbDTXqnBdBmON0f5wcu0cYQ96ndxknt6Os=";
              };

              cargoHash = "sha256-jh2iOnKHhSWMv5ICbBytVTUloKIYhAPNdHJW7vrFfKY=";

              meta = with pkgs.lib; {
                description = "A Bash environment plugin for Nushell";
                homepage = "https://github.com/tesujimath/nu_plugin_bash_env";
                license = licenses.mit;
                maintainers = [ maintainers.tailhook ];
              };

              buildInputs = [ pkgs.makeWrapper ];

              postFixup = ''
                wrapProgram $out/bin/nu_plugin_bash_env --set NU_PLUGIN_BASH_ENV_SCRIPT ${nu_plugin_bash_env_script}/bin/nu_plugin_bash_env_script
              '';
            };
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

          packages.default = nu_plugin_bash_env;
        }
      );
}
