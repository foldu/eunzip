{
  description = "An unzip thing.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, naersk, flake-utils }@inputs: {
    overlay = final: prev: {
      eunzip =
        let
          pkgs = nixpkgs.legacyPackages.${prev.system};
          naersk-lib = naersk.lib."${prev.system}".override {
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
    };
  } // flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        overlays = [ self.overlay ];
        inherit system;
      };
    in
    {
      defaultPackage = pkgs.eunzip;
    }
  );
}
