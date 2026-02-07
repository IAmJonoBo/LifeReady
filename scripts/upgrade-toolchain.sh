#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

toolchain_file="${repo_root}/rust-toolchain.toml"

override_channel=""

usage() {
	cat <<'EOF'
Usage: scripts/upgrade-toolchain.sh [--channel <channel>]

Ensures the repo toolchain is installed and keeps key tools aligned.
If --channel is provided, rust-toolchain.toml is updated to that channel.

Examples:
  scripts/upgrade-toolchain.sh
  scripts/upgrade-toolchain.sh --channel stable
  scripts/upgrade-toolchain.sh --channel 1.85.0
EOF
}

while [[ $# -gt 0 ]]; do
	case "$1" in
	--channel)
		override_channel="${2-}"
		if [[ -z $override_channel ]]; then
			echo "Missing value for --channel" >&2
			usage
			exit 2
		fi
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		echo "Unknown argument: $1" >&2
		usage
		exit 2
		;;
	esac
done

if ! command -v rustup >/dev/null 2>&1; then
	echo "rustup is not installed. Install from https://rustup.rs/ and retry." >&2
	exit 1
fi

channel="stable"
if [[ -f $toolchain_file ]]; then
	channel_from_file="$(grep -E '^[[:space:]]*channel[[:space:]]*=' "$toolchain_file" | head -n 1 | sed -E 's/.*=[[:space:]]*"([^"]+)".*/\1/')"
	if [[ -n $channel_from_file ]]; then
		channel="$channel_from_file"
	fi
fi

if [[ -n $override_channel ]]; then
	channel="$override_channel"
	if [[ -f $toolchain_file ]]; then
		tmp_file="${toolchain_file}.tmp"
		if grep -qE '^[[:space:]]*channel[[:space:]]*=' "$toolchain_file"; then
			sed -E "s/^[[:space:]]*channel[[:space:]]*=.*/channel = \"${channel}\"/" "$toolchain_file" >"$tmp_file"
		else
			cat "$toolchain_file" >"$tmp_file"
			printf '\nchannel = "%s"\n' "$channel" >>"$tmp_file"
		fi
		mv "$tmp_file" "$toolchain_file"
	else
		cat <<EOF >"$toolchain_file"
[toolchain]
channel = "${channel}"
components = ["rustfmt", "clippy"]
EOF
	fi
	echo "Updated rust-toolchain.toml channel to ${channel}"
fi

rustup toolchain install "$channel"
rustup component add rustfmt clippy --toolchain "$channel"

active_toolchain="$(rustup show active-toolchain | awk '{print $1}')"
if [[ $active_toolchain != "${channel}"* ]]; then
	echo "Warning: active toolchain is ${active_toolchain}, expected ${channel}." >&2
	echo "Hint: rust-toolchain.toml will be honored in this repo, or run:" >&2
	echo "  rustup override set ${channel}" >&2
fi

echo "Toolchain ready:"
"$(command -v rustc)" +"$channel" --version
"$(command -v cargo)" +"$channel" --version

if command -v node >/dev/null 2>&1; then
	echo "Node: $(node --version)"
else
	echo "Node.js not found. Install Node 22+ for OpenAPI and markdownlint tooling." >&2
fi

if command -v npm >/dev/null 2>&1; then
	echo "npm: $(npm --version)"
	npm install --prefix "${repo_root}/tools/openapi"
	npm install --prefix "${repo_root}/tools/markdownlint"
else
	echo "npm not found. Skipping OpenAPI and markdownlint installs." >&2
fi

if command -v flutter >/dev/null 2>&1; then
	if command -v brew >/dev/null 2>&1; then
		if brew list --cask flutter >/dev/null 2>&1; then
			brew upgrade --cask flutter
		fi
	fi
	flutter upgrade
	flutter --version | head -n 1
	if [[ -f "${repo_root}/apps/lifeready_flutter/pubspec.yaml" ]]; then
		pub_output=""
		pub_status=0
		if ! pub_output=$(cd "${repo_root}/apps/lifeready_flutter" && flutter pub get 2>&1); then
			pub_status=$?
		fi
		echo "$pub_output" | sed -E \
			-e '/\(.* available\)/d' \
			-e '/newer versions incompatible with dependency constraints/d' \
			-e '/flutter pub outdated/d'
		if [[ $pub_status -ne 0 ]]; then
			exit $pub_status
		fi
	fi
else
	echo "Flutter not found. Install Flutter to generate tokens and run Flutter checks." >&2
fi

if command -v dart >/dev/null 2>&1; then
	dart --version
else
	echo "Dart not found. Install Dart SDK (or Flutter) for token generation." >&2
fi

if command -v docker >/dev/null 2>&1; then
	docker --version
	if command -v docker-compose >/dev/null 2>&1; then
		docker-compose --version
	else
		if docker compose version >/dev/null 2>&1; then
			docker compose version
		else
			echo "Docker Compose not found. Install Docker Desktop or Compose plugin." >&2
		fi
	fi
else
	echo "Docker not found. Install Docker to run dev services." >&2
fi
