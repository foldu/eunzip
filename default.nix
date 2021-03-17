{ rustc, cargo, callPackage }:

let
  sources = import ./nix/sources.nix;
  naersk = callPackage sources.naersk {
    inherit rustc cargo;
  };
  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    ./.;
in
naersk.buildPackage {
  inherit src;
}
