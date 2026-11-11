task("test", "cargo test --workspace --all-features --locked")
task("lint", "cargo clippy --workspace --all-targets --all-features -- -D warnings")
task("fmt", "cargo fmt --all")
task("fmt:check", "cargo fmt --all --check")

task("check", function()
	run("fmt:check")
	run("lint")
	run("test")
end)

task("precommit", function()
	run("fmt")
	run("check")
end)

task("ci", function()
	run("check")
end)
