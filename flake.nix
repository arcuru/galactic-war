{
  description = "galactic-war";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      # Needed because rust-overlay, normally used by crane, doesn't have llvm-tools for coverage
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    advisory-db = {
      # Rust dependency security advisories
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {self, ...} @ inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs {
        inherit system;
      };

      inherit (pkgs) lib;

      # Use the stable rust tools from fenix
      fenixStable = inputs.fenix.packages.${system}.stable;
      rustSrc = fenixStable.rust-src;
      toolChain = fenixStable.completeToolchain;

      # Use the toolchain with the crane helper functions
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolChain;

      # Clean the src to only have the Rust-relevant files
      # src = let
      #   # We need to keep the yaml files because they are used for the build defaults
      #   yamlFilter = path: _type: builtins.match ".*defaults.yaml$" path != null;
      #   yamlOrCargo = path: type:
      #     (yamlFilter path type) || (craneLib.filterCargoSources path type);
      # in
      #   lib.cleanSourceWith {
      #     src = craneLib.path ./.;
      #     filter = yamlOrCargo;
      #   };
      src = ./.;

      # Common arguments for mkCargoDerivation, a helper for the crane functions
      # Arguments can be included here even if they aren't used, but we only
      # place them here if they would otherwise show up in multiple places
      commonArgs = {
        inherit src cargoArtifacts;
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [
          openssl
          sqlite
        ];
      };

      # Build only the cargo dependencies so we can cache them all when running in CI
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # Build the actual crate itself, reusing the cargoArtifacts
      galactic-war = craneLib.buildPackage (commonArgs
        // {
          doCheck = false; # Tests are run as a separate step
          meta.mainProgram = "galactic-war";
        });
    in {
      checks =
        {
          # Build the final package as part of `nix flake check` for convenience
          inherit (self.packages.${system}) galactic-war;

          # Run clippy (and deny all warnings) on the crate source
          galactic-war-clippy =
            craneLib.cargoClippy
            (commonArgs
              // {
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });

          # Check docs build successfully
          galactic-war-doc = craneLib.cargoDoc commonArgs;

          # Check formatting
          galactic-war-fmt = craneLib.cargoFmt commonArgs;

          # Run tests with cargo-nextest
          # Note: This provides limited value, as tests are already run in the build
          galactic-war-nextest = craneLib.cargoNextest commonArgs;

          # Audit dependencies
          crate-audit = craneLib.cargoAudit (commonArgs
            // {
              inherit (inputs) advisory-db;
              # Ignoring RUSTSEC-2023-0071 (RSA Marvin Attack) as it's a transitive dependency
              # through sqlx-mysql that we cannot easily fix. The vulnerability requires specific
              # conditions to exploit and is medium severity.
              cargoAuditExtraArgs = "--ignore RUSTSEC-2023-0071";
            });
        }
        // lib.optionalAttrs (system == "x86_64-linux") {
          # Check code coverage with tarpaulin runs
          galactic-war-tarpaulin = craneLib.cargoTarpaulin commonArgs;
        }
        // {
          # Run formatting checks before commit
          # Can be run manually with `pre-commit run -a`
          pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
            src = ./.;
            tools.rustfmt = toolChain;
            hooks = {
              alejandra.enable = true; # Nix formatting
              prettier = {
                enable = true;
                excludes = [
                  "docs/book/\\.html"
                  "docs/mermaid.min.js"
                  "docs/theme/*"
                ];
              };
              rustfmt.enable = true; # Rust formatting
            };
          };
        };

      packages = rec {
        inherit galactic-war;
        default = galactic-war;
      };

      apps = rec {
        default = galactic-war;
        galactic-war = inputs.flake-utils.lib.mkApp {
          drv = self.packages.${system}.galactic-war;
        };
      };

      devShells.default = pkgs.mkShell {
        name = "galactic-war";
        shellHook = ''
          ${self.checks.${system}.pre-commit-check.shellHook}
          echo ---------------------
          task --list
          echo ---------------------
        '';

        # Include the packages from the defined checks and packages
        inputsFrom =
          (builtins.attrValues self.checks.${system})
          ++ (builtins.attrValues self.packages.${system});

        nativeBuildInputs = with pkgs; [
          act # For running Github Actions locally
          alejandra
          deadnix
          git-cliff
          go-task
          gum # Pretty printing in scripts
          nodePackages.prettier
          statix

          mdbook
          mdbook-mermaid

          # Code coverage
          cargo-tarpaulin
        ];

        # Many tools read this to find the sources for rust stdlib
        RUST_SRC_PATH = "${rustSrc}/lib/rustlib/src/rust/library";
      };
    })
    // {
      overlays.default = final: prev: {
        galactic-war = self.packages.${final.system}.galactic-war;
      };
    };
}
