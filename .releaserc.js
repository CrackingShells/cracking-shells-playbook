module.exports = {
  branches: ["main"],
  plugins: [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    ["@semantic-release/changelog", {
      changelogFile: "skills/${name}/CHANGELOG.md"
    }],
    ["@semantic-release/exec", {
      prepareCmd: "uv run tools/package_skill.py skills/${name} dist/"
    }],
    ["@semantic-release/github", {
      assets: [{ path: "dist/${name}.skill", label: "${name} skill package" }]
    }],
    ["@semantic-release/git", {
      assets: ["skills/${name}/package.json", "skills/${name}/CHANGELOG.md"],
      message: "chore(release): ${name}@${nextRelease.version} [skip ci]"
    }]
  ]
}
