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
        folders = [
          "linux-x64-napi-v6-glibc"
          "linux-x64-napi-v6-musl"
          "linux-arm-napi-v6-glibc"
          "win32-x64-napi-v6-unknown"
          "darwin-x64-napi-v6-unknown"
        ];
        env = rec {
          NAME = "skia-canvas";
          REPO = "samizdatco/skia-canvas";
          VERSION = "v1.0.1";
          NEREID = "@koishijs-assets/skia-canvas";
          URL_PREFIX = "https://skia-canvas.s3.us-east-1.amazonaws.com/${VERSION}";
          FOLDERS = lib.concatStringsSep " " folders;
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
          } {
            name = "copy";
            help = "copy dependencies to src";
            command = "cd ${env.NAME} && ./copy.sh";
          }];
        };
      };
    };
}
