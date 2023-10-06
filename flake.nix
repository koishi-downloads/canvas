{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs@{ flake-parts, ... }: let
    in flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devshell.flakeModule ];
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, pkgs, lib, system, ... }: let
        # =======================================================================
        # configuration
        abi = map toString [ 64 67 72 79 83 102 108 111 115 ];
        os = [ "linux-glibc-x64" "darwin-unknown-x64" "win32-unknown-x64" ];
        env = rec {
          NAME = "canvas";
          REPO = "Automattic/node-canvas";
          VERSION = "v2.11.2";
          NODE_ABI_VERSIONS = lib.concatStringsSep " " abi;
          NODE_OS_LIST = lib.concatStringsSep " " os;
          FOLDERS = let
            set = lib.cartesianProductOfSets { inherit abi os; };
            list = map ({ abi, os }: "${NAME}-${VERSION}-node-v${abi}-${os}") set;
          in lib.concatStringsSep " " list;
        };
        # =======================================================================
      in {
        devshells.default = {
          env = [
            { name = "NPM_TOKEN"; eval = "$(cat ~/.config/npm-token/NPM_TOKEN)"; }
          ] ++ lib.mapAttrsToList (name: value: {
            inherit name value;
          }) env;
          packages = with pkgs; [ gnutar unzip curl yarn ];
          commands = [{
            name = "init-subtree";
            help = "init subtree ${env.NAME}";
            command = "git subtree --prefix=${env.NAME} add https://github.com/${env.REPO}.git ${env.VERSION} --squash";
          } {
            name = "pull-subtree";
            help = "pull ${env.NAME} with tag ${env.VERSION}";
            command = "git subtree --prefix=${env.NAME} pull https://github.com/${env.REPO}.git ${env.VERSION} --squash";
          } {
            name = "build-nereid";
            command = "cd nereid && ./build.sh";
          } {
            name = "pub-nereid";
            command = "cd nereid && ./pub.sh";
          }];
        };
      };
    };
}
