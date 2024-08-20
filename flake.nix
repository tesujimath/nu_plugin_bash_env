{
  description = "Development environment for nu_plugin_bash_env.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # https://lazamar.co.uk/nix-versions/?channel=nixpkgs-unstable&package=bash
    nixpkgs_bash51.url = "github:NixOS/nixpkgs?rev=79b3d4bcae8c7007c9fd51c279a8a67acfa73a2a";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixpkgs_bash51, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          pkgs_bash51 = import nixpkgs_bash51 {
            inherit system;
          };
          nu_plugin_bash_env = pkgs_bash51.writeShellScriptBin "nu_plugin_bash_env"
            (builtins.replaceStrings [ "jq" "(cat)" " sed " ] [ "${pkgs.jq}/bin/jq" "(${pkgs.coreutils}/bin/cat)" " ${pkgs.gnused}/bin/sed " ]
              (builtins.readFile ./nu_plugin_bash_env));
        in
        with pkgs;
        {
          devShells.default = mkShell {
            nativeBuildInputs = [
              pkgs_bash51.bashInteractive
              jq
            ];
          };
          packages.default = nu_plugin_bash_env;
        }
      );
}
