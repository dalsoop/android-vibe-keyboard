uuid:    "vibe-dev-20260413"
name:    "dev"
version: "1.0.0"
player:  "claude"
role:    "member"
model:   "sonnet"
git: {
    user:  "dal-dev"
    email: "dal-dev@vibe-keyboard.local"
}
tools: ["Read", "Write", "Edit", "Glob", "Grep", "Bash"]
env: {
    DAL_MAX_DURATION: "600"
}
dalcli: {
    own: {
        commands: [
            "git status",
            "git diff",
            "git add *",
            "git commit -m *",
            "cargo build *",
            "cargo test *",
            "cargo check *",
        ]
    }
}
