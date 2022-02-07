{
  description = "An unzip thing.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, naersk, flake-utils }@inputs: flake-utils.lib.eachDefaultSystem (
    system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      eunzip =
        let
          naersk-lib = naersk.lib."${system}".override {
            cargo = pkgs.cargo;
            rustc = pkgs.rustc;
          };
        in
        naersk-lib.buildPackage {
          src = ./.;
          singleStep = true;
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
