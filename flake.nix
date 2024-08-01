{
  inputs.nixpkgs.url = "nixpkgs";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    version = builtins.substring 0 7 self.lastModifiedDate;

    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    forAllSystems = nixpkgs.lib.genAttrs systems;
    nixpkgsFor = forAllSystems (system: import nixpkgs {inherit system;});

    packageFn = pkgs:
      pkgs.rustPlatform.buildRustPackage {
        pname = "gay-onboarding";
        inherit version;

        src = builtins.path {
          name = "source";
          path = ./.;
        };

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        separateDebugInfo = true;
      };
  in {
    packages = forAllSystems (s: let
      pkgs = nixpkgsFor.${s};
    in rec {
      gay-onboarding = packageFn pkgs;
      default = gay-onboarding;
    });

    devShells = forAllSystems (s: let
      pkgs = nixpkgsFor.${s};
      inherit (pkgs) mkShell;
    in {
      default = mkShell {
        packages = with pkgs; [rustc cargo rustfmt];
      };
    });

    dockerImage = forAllSystems (s: let
      pkgs = nixpkgsFor.${s};
    in pkgs.dockerTools.buildImage {
      name = "gay-onboarding";
      tag = "latest";
      config = {
        Cmd = ["${packageFn pkgs}/bin/gay-onboarding"];
      };
    });
  };
}