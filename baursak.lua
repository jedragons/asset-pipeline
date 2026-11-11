task("test", "cargo test --workspace --all-features --locked")
task("lint", "cargo clippy --workspace --all-targets --all-features -- -D warnings")
task("fmt", "cargo fmt --all --check")

task("ci", function()
	run("test")
	run("lint")
end)
