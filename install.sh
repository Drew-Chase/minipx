#!/usr/bin/env bash
# Exit immediately on errors (-e), treat unset variables as errors (-u), and
# fail a pipeline if any command errors (-o pipefail).
set -euo pipefail

binary_path=/usr/local/bin/minipx
config_path=/etc/minipx

# This script installs the minipx on a Linux system.
# It checks for necessary tools, downloads the latest release, extracts it,
# and sets up a systemd service to run the panel.
# Usage: Run this script as root or with sudo to install the minipx.
# Ensure the script is run as root or with sudo.
# Example: sudo bash install-minipx.sh
# To uninstall the panel you can run install-minipx.sh with the --uninstall flag.
if [[ "${1:-}" == "--uninstall" ]]; then
  echo "Uninstalling minipx..."
  sudo systemctl stop minipx || true
  sudo systemctl disable minipx || true
  sudo rm -f /etc/systemd/system/minipx.service
  sudo systemctl daemon-reload
  sudo rm -rf $binary_path
  sudo rm -rf $config_path
  echo "minipx uninstalled."
  exit 0
fi

echo "Checking for available download/extract tools..."

# Choose a downloader: prefer curl, fallback to wget. If neither exists, abort.
downloader=""
if command -v curl >/dev/null 2>&1; then
  downloader="curl -fsSL" # -f: fail on HTTP errors, -sS: silent with errors, -L: follow redirects
elif command -v wget >/dev/null 2>&1; then
  downloader="wget -qO-"  # -q: quiet, -O-: write to stdout
else
  echo "Error: Need either curl or wget to download files, but neither is installed."
  echo "Please install curl or wget and re-run."
  exit 1
fi

# Choose a .zip extractor: prefer unzip, fallback to bsdtar or 7z. If none exists, abort.
extractor=""
if command -v unzip >/dev/null 2>&1; then
  extractor="unzip -o"         # -o: overwrite existing files without prompting
elif command -v bsdtar >/dev/null 2>&1; then
  extractor="bsdtar -xf"       # -x: extract, -f: file
elif command -v 7z >/dev/null 2>&1; then
  extractor="7z x -y"          # x: extract with full paths, -y: assume Yes on all queries
else
  echo "Error: Need an extractor for .zip files (unzip, bsdtar, or 7z), but none is installed."
  echo "Please install one of them and re-run."
  exit 1
fi

echo "Downloading minipx artifacts..."

# GitHub API endpoint for repository releases (returns JSON)
releases_url="https://api.github.com/repos/Drew-Chase/minipx/releases"

# Fetch the releases JSON; if it fails, abort.
json="$($downloader "$releases_url")" || { echo "Failed to fetch releases info."; exit 1; }

# Parse the JSON using awk/sed to find the browser_download_url that matches minipx-linux-x64.zip
# Note: This avoids requiring jq by doing a simple line-based search.
download_url="$(
  printf "%s" "$json" \
  | tr -d '\r' \
  | awk '
      /"name":/ { name=$0 }
      /"browser_download_url":/ {
        url=$0
        if (name ~ /minipx-linux-x64\.zip"/) {
          print url
          exit
        }
      }
    ' \
  | sed -n 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p'
)"

# Ensure we found a matching asset URL.
if [ -z "${download_url:-}" ]; then
  echo "Error: Could not find download URL for minipx-linux-x64.zip in the latest release."
  exit 1
fi

echo "Fetching: $download_url"

# Download the ZIP to a local file using the previously selected tool.
if [[ "$downloader" == curl* ]]; then
  curl -fLso minipx.zip "$download_url" # -f: fail on server errors, -L: follow redirects, -s: silent, -o: output file
else
  wget -qO minipx.zip "$download_url"   # -q: quiet, -O: output file
fi

echo "Extracting to ${binary_path}"

# Extract based on the chosen extractor. Suppress noisy output where possible.
if [[ "$extractor" == "unzip -o" ]]; then
  unzip -o minipx.zip -d "$(dirname "$binary_path")" >/dev/null
elif [[ "$extractor" == "bsdtar -xf" ]]; then
  bsdtar -xf minipx.zip -C "$(dirname "$binary_path")"
else
  7z x -y -o"$(dirname "$binary_path")" minipx.zip >/dev/null
fi

# Remove the ZIP after successful extraction to save space.
rm -f minipx.zip

# Work inside the extracted directory for the remainder of setup.
cd "$(dirname "$binary_path")"
# Ensure the minipx binary is executable.
chmod +x "$(basename "$binary_path")"

mkdir "$config_path" || true

# Create a systemd service unit content pointing to the extracted binary in the current directory.
# Uses root user/group and restarts automatically on failure.
service_text="
[Unit]
Description=minipx - A simple, configurable TCP/IP reverse proxy
Wants=network-online.target
After=network-online.target
[Service]
Type=simple
User=root
Group=root
ExecStart=${binary_path} -vw
WorkingDirectory=${config_path}
Restart=always
RestartSec=10
[Install]
WantedBy=multi-user.target
"

echo "Creating systemd service file..."

# Write the systemd unit, set correct permissions, reload systemd, and start the service.
echo "$service_text" | sudo tee /etc/systemd/system/minipx.service >/dev/null
sudo chmod 644 /etc/systemd/system/minipx.service
sudo systemctl daemon-reload
sudo systemctl start minipx
sudo systemctl enable minipx
echo "Systemd service 'minipx' created and started."
echo "Edit the configuration files in ${config_path} to customize minipx."
echo "You can check its status with: sudo systemctl status minipx"
echo "To stop the service, use: sudo systemctl stop minipx"
echo "To disable the service from starting on boot, use: sudo systemctl disable minipx"
echo "To view logs, use: journalctl -u minipx -f"
echo "minipx installation complete."

echo "Done. Service 'minipx' started."
