## spawning-agent-plugins [1.0.1](https://github.com/CrackingShells/cracking-shells-playbook/compare/spawning-agent-plugins@1.0.0...spawning-agent-plugins@1.0.1) (2026-09-16)


### Bug Fixes

* **spawning-agent-plugins:** stop advising --force on a file it refuses to overwrite ([a62a69a](https://github.com/CrackingShells/cracking-shells-playbook/commit/a62a69a9171685b65422522c3e724af9d3a0ffd7))

# spawning-agent-plugins 1.0.0 (2026-09-16)


### Bug Fixes

* **spawning-agent-plugins:** assert every spec-declared plugin has a complete root ([9b00c03](https://github.com/CrackingShells/cracking-shells-playbook/commit/9b00c03722967c77607eb83c53ab750a9e0d1b4b))
* **spawning-agent-plugins:** fall back loudly on a missing version_from target ([926c441](https://github.com/CrackingShells/cracking-shells-playbook/commit/926c4413f1b861d9054f6df8dd5b3256ba25f8b5))
* **spawning-agent-plugins:** key-scope the regeneration guard's allowlist ([57e94ee](https://github.com/CrackingShells/cracking-shells-playbook/commit/57e94ee1defd1b991372be0d870396a5a89ca982))
* **spawning-agent-plugins:** move the example spec to hub mode ([e2e652e](https://github.com/CrackingShells/cracking-shells-playbook/commit/e2e652e58dc3124db0f6e349729b8018c0de9e9a))
* **spawning-agent-plugins:** never regenerate an existing dev README ([e100f32](https://github.com/CrackingShells/cracking-shells-playbook/commit/e100f3257f969b9be658f886f02021f62a73021f))
* **spawning-agent-plugins:** prune vendored dirs from plugin-root discovery ([f6a2e54](https://github.com/CrackingShells/cracking-shells-playbook/commit/f6a2e5471bd355a1772e07486d307bd25cd99267))
* **spawning-agent-plugins:** read dev-plugin identity from the spec, not tree shape ([ae48259](https://github.com/CrackingShells/cracking-shells-playbook/commit/ae48259b93cd3e8c4403b33bbe599abe2590451b))
* **spawning-agent-plugins:** record the hub repository instead of inferring it ([d11b822](https://github.com/CrackingShells/cracking-shells-playbook/commit/d11b822541313f820c5a7a760f9279ea45449eb6))
* **spawning-agent-plugins:** scope dev-plugin discrimination to claude-only+nested ([4e2a2c3](https://github.com/CrackingShells/cracking-shells-playbook/commit/4e2a2c334a53f046831a9ab3b4f31559a4abea37))
* **spawning-agent-plugins:** separate advisory notes from problems ([b622141](https://github.com/CrackingShells/cracking-shells-playbook/commit/b6221419ddb5487ab14e98a364f248d56c86e215))


### Features

* **spawning-agent-plugins:** accept several plugins in one spec ([c77f114](https://github.com/CrackingShells/cracking-shells-playbook/commit/c77f1140ca2bac783506df5c6014eb726ce5aeb7))
* **spawning-agent-plugins:** add the release marker and config ([f93ab19](https://github.com/CrackingShells/cracking-shells-playbook/commit/f93ab1929b10048c6b4fa5278ad6288a054c9ad6))
* **spawning-agent-plugins:** add the skill to the playbook ([eabeab2](https://github.com/CrackingShells/cracking-shells-playbook/commit/eabeab209df3261076da6ff2b6ead287aec4a370))
* **spawning-agent-plugins:** express Codex through the com.openai extensions namespace ([452c926](https://github.com/CrackingShells/cracking-shells-playbook/commit/452c926c3d10bae3d9478ac5b1715f985a1a76d6))
* **spawning-agent-plugins:** support marketplaces owned by a hub repository ([c91cde6](https://github.com/CrackingShells/cracking-shells-playbook/commit/c91cde65cc4ec40903146a1e1c110ef14cf207ab))
* **spawning-agent-plugins:** validate sibling plugins and the extensions shape ([6257554](https://github.com/CrackingShells/cracking-shells-playbook/commit/62575544edfa5584f0e2e43c17074515bfedaf8e))
* **spawning-agent-plugins:** write one manifest set per declared plugin ([35071fb](https://github.com/CrackingShells/cracking-shells-playbook/commit/35071fbdc48655c7ca666904606e82c261a8238c))
