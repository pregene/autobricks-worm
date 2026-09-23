#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "linux-test.sh must run on Linux" >&2
  exit 1
fi

if [[ ! -e /dev/fuse ]]; then
  if [[ "$(id -u)" -eq 0 ]] && command -v modprobe >/dev/null 2>&1; then
    modprobe fuse || true
  elif command -v sudo >/dev/null 2>&1 && command -v modprobe >/dev/null 2>&1; then
    sudo -n modprobe fuse 2>/dev/null || true
  fi
fi

if [[ ! -e /dev/fuse ]]; then
  echo "/dev/fuse is not available. Load the fuse kernel module first: sudo modprobe fuse" >&2
  exit 1
fi

if ! command -v fusermount3 >/dev/null 2>&1 && ! command -v fusermount >/dev/null 2>&1; then
  echo "fusermount3 or fusermount is required for FUSE tests" >&2
  exit 1
fi

RETAINED_STORAGE="${AB_WORM_STORAGE:-$ROOT/worm-storage}"
RETAINED_MOUNT="${AB_WORM_MOUNT:-$ROOT/worm-mount}"
AB_WORM_STORAGE=""
AB_WORM_MOUNT=""
MOUNT_PID=""
LOG_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/ab-worm-linux-test-logs.XXXXXX")"
FAILED=0
RUN_ID="$(date +%Y%m%d-%H%M%S)-$$"
AUDIT_FILE="audit-$RUN_ID.log"
REPORT_DIR="reports-$RUN_ID"
REPORT_FILE="$REPORT_DIR/day1.log"
EMPTY_DIR="empty-dir-$RUN_ID"
DELETE_FILE="delete-$RUN_ID.log"

cleanup() {
  set +e
  if [[ -n "$MOUNT_PID" ]]; then
    "$ROOT/bin/ab-worm" unmount "$AB_WORM_MOUNT" >/dev/null 2>&1 || true
    wait "$MOUNT_PID" >/dev/null 2>&1 || true
    MOUNT_PID=""
  fi
  if [[ "$FAILED" -eq 0 ]]; then
    rm -rf "$LOG_ROOT"
  else
    echo "Logs kept at: $LOG_ROOT" >&2
  fi
}
trap cleanup EXIT INT TERM

fail() {
  FAILED=1
  echo "FAIL: $*" >&2
  exit 1
}

section() {
  echo
  echo "== $* =="
}

ok() {
  printf "  [ok] %s\n" "$*"
}

run_quiet() {
  local label="$1"
  shift
  local logfile="$LOG_ROOT/${label//[^A-Za-z0-9_.-]/_}.log"
  printf "  [run] %s\n" "$label"
  if "$@" >"$logfile" 2>&1; then
    printf "  [ok]  %s\n" "$label"
  else
    printf "  [!!]  %s\n" "$label"
    sed -n '1,160p' "$logfile" >&2
    fail "$label failed (log: $logfile)"
  fi
}

assert_file_contains() {
  local file="$1"
  local expected="$2"
  local actual
  actual="$(cat "$file")"
  [[ "$actual" == "$expected" ]] || fail "$file content mismatch"
}

assert_fails() {
  local label="$1"
  shift
  local stdout="$LOG_ROOT/${label//[^A-Za-z0-9_.-]/_}.out"
  local stderr="$LOG_ROOT/${label//[^A-Za-z0-9_.-]/_}.err"
  set +e
  "$@" >"$stdout" 2>"$stderr"
  local status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    cat "$stdout"
    cat "$stderr" >&2
    fail "$label unexpectedly succeeded"
  fi
  ok "$label rejected"
}

wait_for_mount() {
  local mountpoint="$1"
  for _ in {1..50}; do
    if findmnt -T "$mountpoint" | grep -q "ab-worm"; then
      return 0
    fi
    if [[ -n "$MOUNT_PID" ]] && ! kill -0 "$MOUNT_PID" 2>/dev/null; then
      wait "$MOUNT_PID" || true
      sed -n '1,160p' "$LOG_ROOT/mount.log" >&2 || true
      fail "mount process exited before becoming ready"
    fi
    sleep 0.1
  done
  findmnt -T "$mountpoint" || true
  fail "mount did not become ready"
}

mount_worm() {
  local retain_days="$1"
  mkdir -p "$AB_WORM_STORAGE" "$AB_WORM_MOUNT"
  "$ROOT/bin/ab-worm" mount "$AB_WORM_STORAGE" "$AB_WORM_MOUNT" --retain "$retain_days" >"$LOG_ROOT/mount.log" 2>&1 &
  MOUNT_PID=$!
  wait_for_mount "$AB_WORM_MOUNT"
  ok "mounted retain=$retain_days"
}

unmount_worm() {
  run_quiet "unmount" "$ROOT/bin/ab-worm" unmount "$AB_WORM_MOUNT"
  wait "$MOUNT_PID" >/dev/null 2>&1 || true
  MOUNT_PID=""
}

section "Build"
run_quiet "cargo build --release" cargo build --release --locked
mkdir -p bin
cp target/release/ab-worm bin/ab-worm
chmod +x bin/ab-worm
VERSION_OUTPUT="$(bin/ab-worm --version)"
ok "$VERSION_OUTPUT"

section "Rust Tests"
run_quiet "cargo test --locked" cargo test --locked

section "Retained Mount: --retain 365"
AB_WORM_STORAGE="$RETAINED_STORAGE"
AB_WORM_MOUNT="$RETAINED_MOUNT"
mount_worm 365

printf 'first record\n' >"$AB_WORM_MOUNT/$AUDIT_FILE"
printf 'second record\n' >>"$AB_WORM_MOUNT/$AUDIT_FILE"
assert_file_contains "$AB_WORM_MOUNT/$AUDIT_FILE" $'first record\nsecond record'
ok "file create + append + read"

grep -q '"lock_offset": 27' "$AB_WORM_MOUNT/$AUDIT_FILE.meta" || fail "metadata lock_offset mismatch"
grep -q '"retain_until"' "$AB_WORM_MOUNT/$AUDIT_FILE.meta" || fail "metadata retain_until missing"
ok "metadata visible"

mkdir "$AB_WORM_MOUNT/$REPORT_DIR"
printf 'daily record\n' >"$AB_WORM_MOUNT/$REPORT_FILE"
printf 'follow-up\n' >>"$AB_WORM_MOUNT/$REPORT_FILE"
assert_file_contains "$AB_WORM_MOUNT/$REPORT_FILE" $'daily record\nfollow-up'
grep -q '"lock_offset": 23' "$AB_WORM_MOUNT/$REPORT_FILE.meta" || fail "nested metadata lock_offset mismatch"
ok "directory + nested file append"

mkdir "$AB_WORM_MOUNT/$EMPTY_DIR"
ok "empty directory create"
assert_fails "overwrite before LOCK" bash -c "printf 'replace\n' > '$AB_WORM_MOUNT/$AUDIT_FILE'"
assert_fails "rename immutable path" mv "$AB_WORM_MOUNT/$AUDIT_FILE" "$AB_WORM_MOUNT/renamed-$AUDIT_FILE"
assert_fails "rename immutable directory path" mv "$AB_WORM_MOUNT/$REPORT_DIR" "$AB_WORM_MOUNT/renamed-$REPORT_DIR"
assert_fails "remove retained directory" rmdir "$AB_WORM_MOUNT/$EMPTY_DIR"
assert_fails "reserved metadata directory" mkdir "$AB_WORM_MOUNT/reserved.meta"
assert_fails "nested reserved metadata directory" mkdir "$AB_WORM_MOUNT/$REPORT_DIR/nested.meta"
assert_fails "metadata write" bash -c "printf x >> '$AB_WORM_MOUNT/$AUDIT_FILE.meta'"
assert_fails "retained delete" rm "$AB_WORM_MOUNT/$AUDIT_FILE"

[[ -f "$AB_WORM_MOUNT/$AUDIT_FILE" ]] || fail "retained data file disappeared"
[[ -f "$AB_WORM_MOUNT/$AUDIT_FILE.meta" ]] || fail "retained metadata file disappeared"
[[ -d "$AB_WORM_MOUNT/$REPORT_DIR" ]] || fail "reports directory disappeared"
[[ -d "$AB_WORM_MOUNT/$EMPTY_DIR" ]] || fail "empty-dir directory disappeared"
ok "retained entries preserved after rejected operations"

unmount_worm
run_quiet "verify $AUDIT_FILE" bin/ab-worm storage "$AB_WORM_STORAGE" verify "$AUDIT_FILE"
run_quiet "verify $REPORT_FILE" bin/ab-worm storage "$AB_WORM_STORAGE" verify "$REPORT_FILE"

section "Zero-Retention Delete: --retain 0"
AB_WORM_STORAGE="$RETAINED_STORAGE"
AB_WORM_MOUNT="$RETAINED_MOUNT"
mount_worm 0

printf 'delete me\n' >"$AB_WORM_MOUNT/$DELETE_FILE"
[[ -f "$AB_WORM_MOUNT/$DELETE_FILE.meta" ]] || fail "metadata file was not created"
rm "$AB_WORM_MOUNT/$DELETE_FILE"
[[ ! -e "$AB_WORM_MOUNT/$DELETE_FILE" ]] || fail "$DELETE_FILE still exists in mount"
[[ ! -e "$AB_WORM_MOUNT/$DELETE_FILE.meta" ]] || fail "$DELETE_FILE.meta still exists in mount"
[[ ! -e "$AB_WORM_STORAGE/$DELETE_FILE" ]] || fail "$DELETE_FILE still exists in storage"
[[ ! -e "$AB_WORM_STORAGE/$DELETE_FILE.meta" ]] || fail "$DELETE_FILE.meta still exists in storage"
ok "data + metadata deleted"

unmount_worm

echo
echo "Test data kept at: $RETAINED_STORAGE"
echo "Created retained file: $AUDIT_FILE"
echo "Created retained directory: $REPORT_DIR"
echo
echo "PASS: Linux mount workflow completed"
