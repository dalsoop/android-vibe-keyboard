uuid:    "unknown-dev-20260413"
name:    "dev"
version: "1.0.0"
player:  "claude"
role:    "member"
model:   "sonnet"
git: {
	user:  "dal-dev"
	email: "dal-dev@unknown.local"
}

tools: ["Read", "Write", "Edit", "Glob", "Grep", "Bash"]

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
			"npm install",
			"npm run *",
		]
	}
}
