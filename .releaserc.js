module.exports = {
  branches: ["main"],
  plugins: [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    ["@semantic-release/changelog", {
      changelogFile: "skills/${env.SKILL_NAME}/CHANGELOG.md"
    }],
    ["@semantic-release/exec", {
      prepareCmd: "uv run tools/package_skill.py skills/${env.SKILL_NAME} dist/"
    }],
    ["@semantic-release/github", {
      assets: [{ path: "dist/${env.SKILL_NAME}.skill", label: "${env.SKILL_NAME} skill package" }]
    }],
    ["@semantic-release/git", {
      assets: ["skills/${env.SKILL_NAME}/package.json", "skills/${env.SKILL_NAME}/CHANGELOG.md"],
      message: "chore(release): ${env.SKILL_NAME}@${nextRelease.version} [skip ci]"
    }]
  ]
}
