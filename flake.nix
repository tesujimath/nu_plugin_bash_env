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
        in
          with pkgs;
          {
            devShells.default = mkShell {
              nativeBuildInputs = [
                jq
              ];
            };
          }
      );
}
