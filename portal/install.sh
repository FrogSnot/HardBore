#!/bin/bash
# Install HardBore as native XDG Desktop Portal file chooser

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building backend..."
cd "$SCRIPT_DIR"
cargo build --release

sudo cp "$SCRIPT_DIR/target/release/portal" /usr/local/bin/hardbore-portal
sudo chmod +x /usr/local/bin/hardbore-portal

if [ ! -f /usr/local/bin/hardbore ]; then
    echo "Building main binary..."
    cd "$PROJECT_ROOT/src-tauri"
    cargo build --release
    sudo cp "$PROJECT_ROOT/src-tauri/target/release/hardbore" /usr/local/bin/hardbore
    sudo chmod +x /usr/local/bin/hardbore
fi

echo "Registering portal..."
sudo mkdir -p /usr/share/xdg-desktop-portal/portals
sudo cp "$SCRIPT_DIR/hardbore.portal" /usr/share/xdg-desktop-portal/portals/

mkdir -p ~/.local/share/dbus-1/services
cp "$SCRIPT_DIR/org.freedesktop.impl.portal.desktop.hardbore.service" \
   ~/.local/share/dbus-1/services/

pkill -f hardbore-portal 2>/dev/null || true
pkill -f xdg-desktop-portal 2>/dev/null || true
sleep 1

systemctl --user restart xdg-desktop-portal 2>/dev/null || true

echo "Done. HardBore is now your system file picker."
echo "D-Bus will auto-activate the portal service when needed."


