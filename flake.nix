{
  description = "Development environment for nu_plugin_bash_env.";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixpkgs-unstable;
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          nu_plugin_bash_env = pkgs.writeShellScriptBin "nu_plugin_bash_env"
            (builtins.replaceStrings ["jq"] ["${pkgs.jq}/bin/jq"]
              (builtins.readFile ./nu_plugin_bash_env));
        in
          with pkgs;
          {
            devShells.default = mkShell {
              nativeBuildInputs = [
                bashInteractive
                jq
              ];
            };
            packages.default = nu_plugin_bash_env;
          }
      );
}
