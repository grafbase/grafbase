#!/usr/bin/env bash

set -euo pipefail

# cd to project directory
cli_dir="$(dirname "$(dirname "$(realpath "$0")")")"
cd "$cli_dir"

branch="main"
ci_assets="$(grep 'ASSETS_VERSION:' "$cli_dir/../.github/workflows/cli.yml" | cut -d'/' -f2 | tr -d \')"
latest_assets="latest"
assets=""

show_help() {
	printf "%s [-b branch] [-a assets] [-f]\n" "$(basename "$0")"
	printf "  -h/--help   Show this message.\n"
	printf "  -b/--branch Branch of api repo. Defaults to '%s'.\n" "$branch"
	printf "  -a/--assets Assets version to load. Defaults to '%s' ('%s' for 'main' branch).\n" "$latest_assets" "$ci_assets"
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
if [[ -z "$assets" ]]; then
	if [[ "$branch" == "main" ]]; then
		printf "Loading latest assets used by the CI...\n"
		assets=$ci_assets
	else
		assets=$latest_assets
	fi
fi
if [[ "$branch" == "main" ]]; then
	url="https://assets.grafbase.com/cli/release/$assets.tar.gz"
else
	url="https://assets.grafbase.dev/cli/$branch/$assets.tar.gz"
fi

printf "Cleaning up old assets...\n"
rm -rf "$HOME/.grafbase"
rm -rf "$assets_dir"
mkdir -p "$assets_dir"
cd "$assets_dir"

printf "Loading new assets: %s...\n" "$url"
curl --fail "$url" -o assets.tar.gz
cat >"$assets_dir/.gitignore" <<EOM
*
!.gitignore
EOM

printf "Done!\n"
