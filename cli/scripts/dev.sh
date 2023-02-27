#!/usr/bin/env bash

set -euo pipefail

# cd to project directory
cli_dir="$(dirname "$(dirname "$(realpath "$0")")")"
cd "$cli_dir"

branch="$(git rev-parse --abbrev-ref HEAD)"
assets="latest"
force="" # false

show_help() {
	printf "%s [-b branch] [-a assets] [-f]\n" "$(basename "$0")"
	printf "  -h/--help   Show this message.\n"
	printf "  -f/--force  Whether to force the assets download.\n"
	printf "  -b/--branch Branch of api repo. Defaults to '%s'.\n" "$branch"
	printf "  -a/--assets Assets version to load. Defaults to '%s'.\n" "$assets"
	printf "              See CI upload-cli-assets job, usually '<commit>-<date>'.\n"
	printf "\n"
}

die() {
	printf '%s\n' "$1" >&2
	exit 1
}

if [[ "$#" -ne 0 ]]; then
	while :; do
		case "${1:-}" in
		-h | -\? | --help)
			show_help # Display a usage synopsis.
			exit
			;;
		-b | --branch)
			if [[ -n "${2:-}" ]]; then
				branch="$2"
				shift
			else
				die "Error: $1 requires a non-empty option argument."
			fi
			;;
		-a | --assets)
			if [[ -n "${2:-}" ]]; then
				assets="$2"
				shift
			else
				die "Error: $1 requires a non-empty option argument."
			fi
			;;
		-f | --force)
			force="true"
			;;
		--) # End of all options.
			shift
			break
			;;
		-?*)
			printf 'WARN: Unknown option (ignored): %s\n' "$1" >&2
			;;
		*) # Default case: No more options, so break out of the loop.
			break
			;;
		esac

		shift
	done
	shift $((OPTIND - 1))
fi

assets_dir="$cli_dir/crates/server/assets"
version_file="$assets_dir/.version"
if [[ "$branch" == "main" ]]; then
	url="https://assets.grafbase.com/cli/release/$assets.zip"
else
	url="https://assets.grafbase.dev/cli/$branch/$assets.zip"
fi

if [[ -f "$version_file" ]]; then
	current_version="$(cat "$version_file")"
	printf "Found existing assets: %s\n" "$current_version"
else
	current_version=""
fi

if [[ -n "$force" ]] || [[ "$current_version" != "$url" ]]; then
	printf "Cleaning up old assets...\n"
	rm -rf "$HOME/.grafbase"
	rm -rf "$assets_dir"
	mkdir -p "$assets_dir"
	cd "$assets_dir"

	printf "Loading new assets: %s...\n" "$url"
	curl "$url" -o assets.zip
	unzip assets.zip &>/dev/null
	rm assets.zip
	echo -n "$url" >"$version_file"

	printf "Done!\n"
fi

cat >"$assets_dir/.gitignore" <<EOM
*
!.gitignore
EOM

cd "$cli_dir"
cargo run -- dev
