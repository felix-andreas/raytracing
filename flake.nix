{
  inputs = {
    # base
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # extra
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
      # see: https://github.com/NixOS/nix/issues/5790
      inputs.flake-utils.inputs.systems.follows = "systems";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      # see: https://github.com/NixOS/nix/issues/5790
      inputs.flake-utils.inputs.systems.follows = "systems";
    };
  };

  outputs =
    { self
      # base
    , systems
    , nixpkgs
      # extra
    , crane
    , devshell
    , rust-overlay
    } @ inputs:
    let
      l = inputs.nixpkgs.lib // builtins;
      fs = l.fileset;
      eachSystem = fn: l.genAttrs (import systems) fn;
      flake = (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              devshell.overlays.default
              (import rust-overlay)
            ];
          };
          rust-toolchain = pkgs.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            });
          craneLib = (crane.mkLib pkgs).overrideToolchain rust-toolchain;
          rustFiles = fs.fileFilter (file: file.hasExt "rs") ./.;
          cargoFiles = fs.unions [
            (fs.fileFilter (file: file.name == "Cargo.toml" || file.name == "Cargo.lock") ./.)
          ];
          commonArgs = {
            pname = "crate";
            version = "0.1";
          };
          crateDepsOnly = craneLib.buildDepsOnly (commonArgs // {
            cargoCheckCommandcargo = "check --profile release --all-targets --all-features";
            src = fs.toSource {
              root = ./.;
              fileset = cargoFiles;
            };
          });
          crateClippy = craneLib.cargoClippy (commonArgs // {
            cargoArtifacts = crateDepsOnly;
            cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
            src = fs.toSource {
              root = ./.;
              fileset = fs.unions ([
                cargoFiles
                rustFiles
              ]);
            };
          });
          package = craneLib.buildPackage (commonArgs // {
            pname = "raytracing";
            cargoArtifacts = crateClippy;
            src = fs.toSource {
              root = ./.;
              fileset = fs.unions ([
                cargoFiles
                rustFiles
              ]);
            };
          });
        in
        {
          devShell = pkgs.devshell.mkShell {
            motd = "";
            packages = with pkgs; [
              # Rust
              bacon
              cargo-expand
              cargo-sort
              evcxr
              rust-toolchain
            ];
          };
          check = crateClippy;
          package = package;
          render = pkgs.runCommand "render"
            {
              buildInputs = [ package ];
            }
            ''
              mkdir -p $out
              raytracing -q high -o $out/out.ppm
            '';
        });
    in
    {
      checks = eachSystem (system: { default = (flake system).check; });
      devShells = eachSystem (system: { default = (flake system).devShell; });
      packages = eachSystem (system: {
        default = (flake system).package;
        render = (flake system).render;
      });
    };
}
