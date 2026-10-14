{
  description = "Flake";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/26.05";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    self.submodules = true;
  };

  outputs =
    {
      flake-utils,
      nixpkgs,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        froststrap = pkgs.callPackage ./nix/package.nix { };
      in
      {
        devShells =
          let
            inherit (pkgs.callPackage ./nix/devshell-tools.nix { })
              mkComposedShell
              ;

            extraFrag = pkgs.callPackage ./nix/extra.nix { };
            rustFrag = (pkgs.callPackage ./nix/rustDevShell.nix { }) inputs;
          in
          mkComposedShell [ rustFrag ] { }
          // mkComposedShell [
            rustFrag
            extraFrag
          ] { nameOverride = "default"; };

        packages = {
          debug = pkgs.callPackage ./nix/build.nix { };
          inherit froststrap;
          default = froststrap;
        };

        formatter = (pkgs.callPackage ./nix/formatter.nix { }) inputs;
      }
    );
}
