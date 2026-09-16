/**
 * Shared semantic-release config factory for per-skill releases.
 *
 * Usage in skills/<name>/.releaserc.js:
 *   module.exports = require('../../release-config')('my-skill');
 */
module.exports = function releaseConfig(skillName) {
  return {
    branches: ["main"],
    plugins: [
      "@semantic-release/commit-analyzer",
      "@semantic-release/release-notes-generator",
      ["@semantic-release/changelog", {
        changelogFile: "CHANGELOG.md"
      }],
      ["@semantic-release/exec", {
        prepareCmd: `uv run ../../tools/package_skill.py . ../../dist/`
      }],
      ["@semantic-release/exec", {
        prepareCmd: `uv run ../../tools/set_plugin_version.py ${skillName} \${nextRelease.version}`
      }],
      ["@semantic-release/github", {
        assets: [{ path: `../../dist/${skillName}.skill`, label: `${skillName} skill package` }]
      }],
      ["@semantic-release/git", {
        assets: [
          "package.json",
          "CHANGELOG.md",
          `../../plugins/${skillName}/plugin.json`,
          `../../plugins/${skillName}/.claude-plugin/plugin.json`,
          `../../plugins/${skillName}/skills/${skillName}/**`
        ],
        message: `chore(release): ${skillName}@\${nextRelease.version} [skip ci]`
      }]
    ]
  };
};
