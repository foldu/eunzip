{
  description = "An unzip thing.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        eunzip = craneLib.buildPackage {
          src = self;
          nativeBuildInputs = with pkgs; [
            installShellFiles
          ];
          postInstall = ''
            installShellCompletion target/release/build/pickwp-*/out/pickwp.{fish,bash}
            installShellCompletion --zsh target/release/build/pickwp-*/out/_pickwp
          '';
        };
      in
      {
        defaultPackage = eunzip;
        packages = {
          inherit eunzip;
        };
      }
    );
}
