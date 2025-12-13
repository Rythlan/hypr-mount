#!/bin/sh
# hypr-mount Polkit Rule Manager
# Safe, minimal-privilege setup for passwordless drive mounting
# Usage:
#   ./hypr-mount.sh install    # Install rule (requires sudo once)
#   ./hypr-mount.sh uninstall  # Remove rule (requires sudo)
#   ./hypr-mount.sh check      # Verify installation status
#   ./hypr-mount.sh run        # Run application (default action)

set -eu # Exit on error, unset variable usage

POLKIT_FILE="/etc/polkit-1/rules.d/90-hypr-mount.rules"
CURRENT_USER="${SUDO_USER:-${USER:-$(whoami)}}"
APP_NAME="hypr-mount"

# Validate current user exists
if ! id -u "$CURRENT_USER" >/dev/null 2>&1; then
  echo "[!] ERROR: User '$CURRENT_USER' does not exist on this system"
  echo "    Please run as a valid system user (not root)"
  exit 1
fi

# Validate we're not running as root for app execution
if [ "$(id -u)" = 0 ] && [ "${1:-run}" = "run" ]; then
  echo "[!] ERROR: Do not run application as root"
  echo "    Run as normal user after installing rule:"
  echo "      sudo ./hypr-mount.sh install"
  echo "      ./hypr-mount.sh run"
  exit 1
fi

install_rule() {
  echo "[*] Creating Polkit rule for user: $CURRENT_USER"
  echo "[!] Root access required for this step only"

  # Comprehensive mount permissions (covers all drive types)
  RULE_CONTENT="
polkit.addRule(function(action, subject) {
    const mountActions = [
        'org.freedesktop.udisks2.filesystem-mount',
        'org.freedesktop.udisks2.filesystem-mount-system',
        'org.freedesktop.udisks2.filesystem-mount-other-seat',
        'org.freedesktop.udisks2.filesystem-unmount'
    ];
    
    // Check if the action ID is in the array using .indexOf() > -1
    // This is universally compatible with older JS engines.
    if (mountActions.indexOf(action.id) > -1 && 
        subject.user === '$CURRENT_USER') {
        return polkit.Result.YES;
    }
});
"

  # Atomic write with validation
  printf "%s" "$RULE_CONTENT" |
    sudo tee "$POLKIT_FILE" >/dev/null &&
    sudo chmod 0644 "$POLKIT_FILE" ||
    {
      echo "[✗] FAILED: Could not write Polkit rule" >&2
      exit 1
    }

  echo "[✓] Rule installed: $POLKIT_FILE"

  # Reload polkit with fallbacks
  echo "[*] Reloading authorization service..."
  if sudo systemctl reload polkit.service 2>/dev/null; then
    echo "[✓] Service reloaded successfully"
  elif sudo systemctl restart polkitd.service 2>/dev/null; then
    echo "[✓] Service restarted successfully (fallback)"
  else
    echo "[!] WARNING: Manual reload required"
    echo "    Either log out and back in, or run:"
    echo "      sudo systemctl reload polkit.service || sudo systemctl restart polkitd.service"
  fi

  echo ""
  echo "[!] IMPORTANT: New mounts will now use your user permissions"
  echo "    Existing mounted drives must be unmounted first to take effect"
}

uninstall_rule() {
  if [ -f "$POLKIT_FILE" ]; then
    echo "[*] Removing Polkit rule"
    sudo rm -f "$POLKIT_FILE" ||
      {
        echo "[✗] FAILED: Could not remove rule" >&2
        exit 1
      }
    echo "[✓] Rule uninstalled: $POLKIT_FILE"

    # Reload service after removal
    echo "[*] Reloading authorization service..."
    sudo systemctl reload polkit.service 2>/dev/null ||
      sudo systemctl restart polkitd.service 2>/dev/null ||
      echo "[!] Manual reload required (see install instructions)"
  else
    echo "[!] Rule not found at $POLKIT_FILE"
    exit 1
  fi
}

check_rule() {
  if [ -f "$POLKIT_FILE" ]; then
    echo "[✓] Polkit rule is ACTIVE"
    echo "    Location: $POLKIT_FILE"
    echo "    Permissions: $(stat -c '%a' "$POLKIT_FILE" 2>/dev/null || echo 'unknown')"
    echo ""
    echo "    Rule content:"
    echo "    ----------------------------------------"
    cat "$POLKIT_FILE"
    echo "    ----------------------------------------"
    exit 0
  else
    echo "[!] Polkit rule is NOT INSTALLED"
    echo "    Run: sudo ./install-polkit.sh install"
    exit 1
  fi
}

run_app() {
  echo "[*] Starting $APP_NAME (user context: $CURRENT_USER)"
  echo "=============================================="

  # Build the app if it doesn't exist
  if [ ! -x "target/debug/$APP_NAME" ] && [ ! -x "target/release/$APP_NAME" ]; then
    echo "[*] Building $APP_NAME..."
    cargo build
    if [ $? -ne 0 ]; then
      echo "[✗] ERROR: Failed to build $APP_NAME" >&2
      exit 1
    fi
  fi

  # Run as normal user (never as root) and pass through any arguments
  if [ -x "target/debug/$APP_NAME" ]; then
    exec "./target/debug/$APP_NAME" "$@"
  elif [ -x "target/release/$APP_NAME" ]; then
    exec "./target/release/$APP_NAME" "$@"
  elif command -v "$APP_NAME" >/dev/null 2>&1; then
    exec "$APP_NAME" "$@"
  else
    echo "[✗] ERROR: Application binary not found" >&2
    echo "    Build failed to produce executable." >&2
    exit 1
  fi
}

# Main command router
case "${1:-run}" in
install) install_rule ;;
uninstall) uninstall_rule ;;
check) check_rule ;;
run)
  # Shift to handle arguments passed to the app
  shift
  run_app "$@"
  ;;
*)
  echo "Usage: $0 {install|uninstall|check|run} [args...]"
  echo ""
  echo "  install              : Setup passwordless mounting (requires sudo)"
  echo "  uninstall            : Remove Polkit rule (requires sudo)"
  echo "  check                : Verify installation status"
  echo "  run [args...]        : Start $APP_NAME with optional arguments (default)"
  echo ""
  echo "Examples:"
  echo "  $0 run               : Run normally"
  echo "  $0 run --help        : Show application help"
  echo "  $0 run --auto-mount  : Run with auto mount feature"
  echo ""
  echo "Security notes:"
  echo "  - Rule grants ONLY mount/unmount permissions to $CURRENT_USER"
  echo "  - Does NOT grant format/partition/delete permissions"
  echo "  - Reversible with 'uninstall' command"
  exit 1
  ;;
esac
