{
  description = "Development environment for nu_plugin_bash_env.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # https://lazamar.co.uk/nix-versions/?channel=nixpkgs-unstable&package=jq
    nixpkgs_jq16.url = "github:NixOS/nixpkgs?rev=976fa3369d722e76f37c77493d99829540d43845";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixpkgs_jq16, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          pkgs_jq16 = import nixpkgs_jq16 {
            inherit system;
          };
          nu_plugin_bash_env = pkgs.writeShellScriptBin "nu_plugin_bash_env"
            (builtins.replaceStrings [ "jq" "(cat)" " sed " ] [ "${pkgs_jq16.jq}/bin/jq" "(${pkgs.coreutils}/bin/cat)" " ${pkgs.gnused}/bin/sed " ]
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
