uuid:    "unknown-leader-20260413"
name:    "leader"
version: "1.0.0"
player:  "claude"
role:    "leader"
model:   "opus"
git: {
	user:  "dal-leader"
	email: "dal-leader@unknown.local"
}

// Squad 얇은 라우터 원칙 — tools 는 Read/Glob/Grep (read-only) 만. dal-profile 강제.
// own.commands 의 read-only 인스펙션 (git status/log, ls, cat) 은 허용 — context 수집용.
tools: ["Read", "Glob", "Grep"]

dalcli: {
	own: {
		commands: ["git status", "git log --oneline -*", "ls *", "cat *"]
	}
	can_delegate: {
		commands: ["git push *"]
		to: ["designer", "validator", "dev", "verifier", "deployer"]
	}
}
