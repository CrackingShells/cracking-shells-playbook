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
        changelogFile: `skills/${skillName}/CHANGELOG.md`
      }],
      ["@semantic-release/exec", {
        prepareCmd: `uv run tools/package_skill.py skills/${skillName} dist/`
      }],
      ["@semantic-release/github", {
        assets: [{ path: `dist/${skillName}.skill`, label: `${skillName} skill package` }]
      }],
      ["@semantic-release/git", {
        assets: [`skills/${skillName}/package.json`, `skills/${skillName}/CHANGELOG.md`],
        message: `chore(release): ${skillName}@\${nextRelease.version} [skip ci]`
      }]
    ]
  };
};
