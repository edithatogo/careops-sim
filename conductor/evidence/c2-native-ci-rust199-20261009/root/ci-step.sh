set -euo pipefail
log_dir=".artifacts/c2-native-ci/${EXPECTED_OS}"
mkdir -p "$log_dir"
exclude_file="$(git rev-parse --git-path info/exclude)"
mkdir -p "$(dirname "$exclude_file")"
touch "$exclude_file"
for pattern in '/.artifacts/c2-native-ci/' '/.artifacts/c2-replay-demo/'; do
  if ! grep -Fxq "$pattern" "$exclude_file"; then
    printf '%s\n' "$pattern" >> "$exclude_file"
  fi
done
git rev-parse HEAD | tee "$log_dir/source.log"
task_rustc="$(rustup which --toolchain 1.99.0 rustc)"
task_bin="$(dirname "$task_rustc")"
task_cargo="$(rustup which --toolchain 1.99.0 cargo)"
export RUSTC="$task_rustc" RUSTDOC="$task_bin/rustdoc" PATH="$task_bin:$PATH"
"$task_cargo" -V | tee -a "$log_dir/source.log"
"$task_rustc" -vV | tee -a "$log_dir/source.log"
grep -Fx 'release: 1.99.0' "$log_dir/source.log"
if [ "$EXPECTED_OS" = macos-15 ]; then
  grep -Fx 'host: aarch64-apple-darwin' "$log_dir/source.log"
else
  grep -Fx 'host: x86_64-unknown-linux-gnu' "$log_dir/source.log"
fi
export CARGO_TARGET_DIR="$RUNNER_TEMP/c2-native-ci-target"
rustup run 1.99.0 cargo test --locked -p kairo-ecs-des -p kairo-ecs-abm -p kairo-ecs-calibration --features kairo-ecs-calibration/flow,kairo-ecs-abm/test-support -- --nocapture 2>&1 | tee "$log_dir/flow-tests.log"
"$task_cargo" build --locked -p kairo-ecs-calibration --features flow,kairo-ecs-abm/test-support --example c2_replay_demo 2>&1 | tee "$log_dir/example-build.log"
binary="$CARGO_TARGET_DIR/debug/examples/c2_replay_demo"
test -x "$binary"
shasum -a 256 "$binary" | tee "$log_dir/example-binary.sha256"
checkpoint="$log_dir/checkpoint.json"
test ! -e "$checkpoint"
"$binary" save "$checkpoint" 2>&1 | tee "$log_dir/save.log"
shasum -a 256 "$checkpoint" | tee "$log_dir/checkpoint.sha256"
"$binary" restore "$checkpoint" > "$log_dir/restore-first.json" 2> "$log_dir/restore-first.log"
"$binary" restore "$checkpoint" > "$log_dir/restore-second.json" 2> "$log_dir/restore-second.log"
cmp "$log_dir/restore-first.json" "$log_dir/restore-second.json"
printf '%s\n' 'repeat_restore_output=identical' | tee "$log_dir/repeat-check.log"
sentinel="$log_dir/preexisting-checkpoint.json"
expected="$log_dir/preexisting-checkpoint.expected"
printf '%s\n' 'preexisting sentinel bytes' > "$sentinel"
cp "$sentinel" "$expected"
if "$binary" save "$sentinel" > "$log_dir/sentinel-save.stdout" 2> "$log_dir/sentinel-save.stderr"; then
  status=0
else
  status=$?
fi
test "$status" -eq 1
grep -F 'checkpoint destination already exists' "$log_dir/sentinel-save.stderr"
cmp "$expected" "$sentinel"
printf 'sentinel_save_exit=%s\nexisting_target_rejected=true\nsentinel_bytes_unchanged=true\n' "$status" | tee "$log_dir/sentinel-check.log"
