# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/arcuru/galactic-war/releases/tag/v0.1.0) - 2024-10-13

### Added

- adding storage depot
- adding production data to the system header
- updating the web page building
- updating the web page building
- create and use a blitz game
- improve the web page with expandability in mind
- change how the score is calculated per system
- use production/hour numbers
- Only allow 1 construction prpoject at a time
- add some math for the Resources struct
- rename resources
- add a simple page for building new buildings with the fortress
- serve some very basic html pages
- add a dumb web interface, without pretty pages
- expose event info per island
- add lumber/stone production
- add building details
- retrieve island details
- verify we never go back in time
- add a time needed to build each building
- add build costs for buildings
- switch to using a vec of enums for buildings
- add a production configuration for the goldpit
- read data from a config file
- add very basic events
- add basic building support

### Fixed

- Use a consistent order for the structure names
- handle costs for structures that rely on the multiplier
- removing leftover print statements
- fixing docker hub address
- clippy
- remove the unused POST request handlers
- improving the default world for testing
- fixing activity hover

### Other

- adding basic info to Cargo.toml
- add a link to the test version
- add docker commands to the Taskfile
- bump Cargo.lock
- bump all deps
- formatting
- adding Codeberg sync
- formatting
- updating inselkampf link
- add docker config
- include the binary deps by default
- rename `cli` feature to `bin` for accuracy
- minor fixes to the web interface
- simplify SystemProduction usage
- move resources into a common struct
- use a Coords struct
- rename to Galactic War
- add nicer `task ci` alias
- loosen dep requirements and update
- move out config structs to their own module
- separate dependencies for the binary
- factor out a building_config helper
- move island utilities into it's own module
- Initial commit
