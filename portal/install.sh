#!/bin/bash
# Install HardBore as native XDG Desktop Portal file chooser

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building backend..."
cd "$PROJECT_ROOT/src-tauri"
cargo build --release --bin portal

sudo cp "$PROJECT_ROOT/src-tauri/target/release/portal" /usr/local/bin/hardbore-portal
sudo chmod +x /usr/local/bin/hardbore-portal

if [ ! -f /usr/local/bin/hardbore ]; then
    echo "Building main binary..."
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

pkill -f xdg-desktop-portal 2>/dev/null || true
pkill -f hardbore-portal 2>/dev/null || true
sleep 1

/usr/local/bin/hardbore-portal &

echo "Done. HardBore is now your system file picker. :D"


