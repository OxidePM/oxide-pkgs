# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
#
# Same as nix but with less boilerplate to more easily debug problems
# TODO: add back removed features like logging
# TODO: naming conventions are all over the place

set -eu
set -o pipefail

if [[ -n "${BASH_VERSINFO-}" && "${BASH_VERSINFO-}" -lt 5 ]]; then
    echo "Detected Bash version that isn't supported by oxide-pkgs (${BASH_VERSION})"
    echo "Please install Bash 5 or greater to continue."
    exit 1
fi

shopt -s inherit_errexit

get_all_output_names() {
    echo "$outputs"
}

run_hook() {
    local hook_name="$1"
    shift
    local hooks_slice="${hook_name%_HOOK}_HOOKS[@]"

    local hook
    for hook in "_call_implicit_hook 0 $hook_name" ${!hooks_slice+"${!hooks_slice}"}; do
        _eval "$hook" "$@"
    done

    return 0
}


# Run all hooks with the specified name, until one succeeds (returns a
# zero exit code). If none succeed, return a non-zero exit code.
run_one_hook() {
    local hook_name="$1"
    shift
    local hooks_slice="${hook_name%_HOOK}_HOOKS[@]"

    local hook ret=1
    for hook in "_call_implicit_hook 1 $hook_name" ${!hooks_slice+"${!hooks_slice}"}; do
        if _eval "$hook" "$@"; then
            ret=0
            break
        fi
    done

    return "$ret"
}


_call_implicit_hook() {
    local def="$1"
    local hook_name="$2"
    if declare -F "$hook_name" > /dev/null; then
        "$hook_name"
    elif type -p "$hook_name" > /dev/null; then
        # shellcheck disable=SC1090
        source "$hook_name"
    elif [ -n "${!hook_name:-}" ]; then
        eval "${!hook_name}"
    else
        return "$def"
    fi
}

_eval() {
    if declare -F "$1" > /dev/null 2>&1; then
        "$@" # including args
    else
        eval "$1"
    fi
}


exit_handler() {
    exit_code="$?"
    set +e

    if (( "$exit_code" != 0 )); then
        run_hook FAILURE_HOOK
    else
        run_hook EXIT_HOOK
    fi

    return "$exit_code"
}

trap "exit_handler" EXIT


######################################################################
# Helper functions.
addToSearchPathWithCustomDelimiter() {
    local delimiter="$1"
    local varName="$2"
    local dir="$3"
    if [[ -d "$dir" && "${!varName:+${delimiter}${!varName}${delimiter}}" \
          != *"${delimiter}${dir}${delimiter}"* ]]; then
        export "${varName}=${!varName:+${!varName}${delimiter}}${dir}"
    fi
}

addToSearchPath() {
    addToSearchPathWithCustomDelimiter ":" "$@"
}

# Prepend elements to variable "$1", which may come from an attr.
#
# This is useful in generic setup code, which must (for now) support
# both derivations with and without __structuredAttrs true, so the
# variable may be an array or a space-separated string.
#
# Expressions for individual packages should simply switch to array
# syntax when they switch to setting __structuredAttrs = true.
prependToVar() {
    local -n nameref="$1"
    local useArray=false

    # check if variable already exist and if it does then do extra checks
    if type=$(declare -p "$1" 2> /dev/null); then
        case "${type#* }" in
            -A*)
                echo "prependToVar(): ERROR: trying to use prependToVar on an associative array." >&2
                return 1 ;;
            -a*)
                useArray=true ;;
            *)
                useArray=false ;;
        esac
    fi

    shift

    if $useArray; then
        nameref=( "$@" ${nameref+"${nameref[@]}"} )
    else
        nameref="$* ${nameref-}"
    fi
}

# Same as above
appendToVar() {
    local -n nameref="$1"
    local useArray=false

    # check if variable already exist and if it does then do extra checks
    if type=$(declare -p "$1" 2> /dev/null); then
        case "${type#* }" in
            -A*)
                echo "appendToVar(): ERROR: trying to use appendToVar on an associative array, use variable+=([\"X\"]=\"Y\") instead." >&2
                return 1 ;;
            -a*)
                useArray=true ;;
            *)
                useArray=false ;;
        esac
    fi

    shift

    if $useArray; then
        nameref=( ${nameref+"${nameref[@]}"} "$@" )
    else
        nameref="${nameref-} $*"
    fi
}

# Accumulate flags from the named variables $2+ into the indexed array $1.
#
# Arrays are simply concatenated, strings are split on whitespace.
# Default values can be passed via name=default.
concatTo() {
    local -
    set -o noglob
    local -n targetref="$1"; shift
    local arg default name type
    for arg in "$@"; do
        IFS="=" read -r name default <<< "$arg"
        local -n nameref="$name"
        if [[ -z "${nameref[*]}" && -n "$default" ]]; then
            targetref+=( "$default" )
        elif type=$(declare -p "$name" 2> /dev/null); then
            case "${type#* }" in
                -A*)
                    echo "concatTo(): ERROR: trying to use concatTo on an associative array." >&2
                    return 1 ;;
                -a*)
                    targetref+=( "${nameref[@]}" ) ;;
                *)
                    if [[ "$name" = *"Array" ]]; then
                        # Reproduces https://github.com/NixOS/nixpkgs/pull/318614/files#diff-7c7ca80928136cfc73a02d5b28350bd900e331d6d304857053ffc9f7beaad576L359
                        targetref+=( ${nameref+"${nameref[@]}"} )
                    else
                        # shellcheck disable=SC2206
                        targetref+=( ${nameref-} )
                    fi
                    ;;
            esac
        fi
    done
}

# Concatenate a list of strings ($2) with a separator ($1) between each element.
# The list can be an indexed array of strings or a single string. A single string
# is split on spaces and then concatenated with the separator.
#
# $ flags="lorem ipsum dolor sit amet"
# $ concatStringsSep ";" flags
# lorem;ipsum;dolor;sit;amet
#
# $ flags=("lorem ipsum" "dolor" "sit amet")
# $ concatStringsSep ";" flags
# lorem ipsum;dolor;sit amet
#
# Also supports multi-character separators;
# $ flags=("lorem ipsum" "dolor" "sit amet")
# $ concatStringsSep " and " flags
# lorem ipsum and dolor and sit amet
concatStringsSep() {
    local sep="$1"
    local name="$2"
    local type
    local IFS
    if type=$(declare -p "$name" 2> /dev/null); then
        local -n nameref="$name"
        case "${type#* }" in
            -A*)
                echo "concatStringsSep(): ERROR: trying to use concatStringsSep on an associative array." >&2
                return 1 ;;
            -a*)
                # \036 is the "record separator" character. We assume that this will never need to be part of
                # an argument string we create here. If anyone ever hits this limitation: Feel free to refactor.
                # To avoid leaking an unescaped rs character when dumping the environment with oxide, we use printf
                # in a subshell.
                IFS="$(printf '\036')" ;;
            *)
                IFS=" " ;;
        esac
        local ifs_separated="${nameref[*]}"
        echo -n "${ifs_separated//"$IFS"/"$sep"}"
    fi
}

# Add $1/lib* into rpaths.
# The function is used in multiple-outputs.sh hook,
# so it is defined here but tried after the hook.
_add_rpath_prefix() {
    export OXIDE_LDFLAGS="-rpath $1/lib ${OXIDE_LDFLAGS-}"
}

# Return success if the specified file is an ELF object.
isELF() {
    local fn="$1"
    local fd
    local magic
    exec {fd}< "$fn"
    read -r -n 4 -u "$fd" magic
    exec {fd}<&-
    if [ "$magic" = $'\177ELF' ]; then return 0; else return 1; fi
}

# Return success if the specified file is a Mach-O object.
isMachO() {
    local fn="$1"
    local fd
    local magic
    exec {fd}< "$fn"
    read -r -n 4 -u "$fd" magic
    exec {fd}<&-

    if [[ "$magic" = $(echo -ne "\xfe\xed\xfa\xcf") || "$magic" = $(echo -ne "\xcf\xfa\xed\xfe") ]]; then
        # MH_MAGIC_64 || MH_CIGAM_64
        return 0;
    elif [[ "$magic" = $(echo -ne "\xfe\xed\xfa\xce") || "$magic" = $(echo -ne "\xce\xfa\xed\xfe") ]]; then
        # MH_MAGIC || MH_CIGAM
        return 0;
    elif [[ "$magic" = $(echo -ne "\xca\xfe\xba\xbe") || "$magic" = $(echo -ne "\xbe\xba\xfe\xca") ]]; then
        # FAT_MAGIC || FAT_CIGAM
        return 0;
    else
        return 1;
    fi
}

# Return success if the specified file is a script (i.e. starts with
# "#!").
isScript() {
    local fn="$1"
    local fd
    local magic
    exec {fd}< "$fn"
    read -r -n 2 -u "$fd" magic
    exec {fd}<&-
    if [[ "$magic" =~ \#! ]]; then return 0; else return 1; fi
}

# printf unfortunately will print a trailing newline regardless
printLines() {
    (( "$#" > 0 )) || return 0
    printf '%s\n' "$@"
}

printWords() {
    (( "$#" > 0 )) || return 0
    printf '%s ' "$@"
}

######################################################################
# Initialisation.

# Set a fallback default value for SOURCE_DATE_EPOCH, used by some build tools
# to provide a deterministic substitute for the "current" time. Note that
# 315532800 = 1980-01-01 12:00:00. We use this date because python's wheel
# implementation uses zip archive and zip does not support dates going back to
# 1970.
export SOURCE_DATE_EPOCH
: "${SOURCE_DATE_EPOCH:=315532800}"


# Wildcard expansions that don't match should expand to an empty list.
# This ensures that, for instance, "for i in *; do ...; done" does the
# right thing.
shopt -s nullglob


# Set up the initial path.
PATH=
HOST_PATH=
for i in $initial_path; do
    if [ "$i" = / ]; then i=; fi
    addToSearchPath PATH "$i/bin"

    # For backward compatibility, we add initial path to HOST_PATH so
    # it can be used in auto patch-shebangs. Unfortunately this will
    # not work with cross compilation.
    if [ -z "${strictDeps-}" ]; then
        addToSearchPath HOST_PATH "$i/bin"
    fi
done

unset i

echo "initial path: $PATH"

# Check that the pre-hook initialised SHELL.
if [ -z "${SHELL:-}" ]; then echo "SHELL not set"; exit 1; fi
BASH="$SHELL"
export CONFIG_SHELL="$SHELL"


# Execute the pre-hook.
if [ -z "${shell:-}" ]; then export shell="$SHELL"; fi
run_hook PRE_HOOK


# TODO: this section must be changed
declare -a pkgsBuildBuild pkgsBuildHost pkgsBuildTarget
declare -a pkgsHostHost pkgsHostTarget
declare -a pkgsTargetTarget

declare -a pkgBuildAccumVars=(pkgsBuildBuild pkgsBuildHost pkgsBuildTarget)
declare -a pkgHostAccumVars=(pkgsHostHost pkgsHostTarget)
declare -a pkgTargetAccumVars=(pkgsTargetTarget)

declare -a pkgAccumVarVars=(pkgBuildAccumVars pkgHostAccumVars pkgTargetAccumVars)


# Hooks
declare -a envBuildBuildHooks envBuildHostHooks envBuildTargetHooks
declare -a envHostHostHooks envHostTargetHooks
declare -a envTargetTargetHooks

declare -a pkgBuildHookVars=(envBuildBuildHook envBuildHostHook envBuildTargetHook)
declare -a pkgHostHookVars=(envHostHostHook envHostTargetHook)
declare -a pkgTargetHookVars=(envTargetTargetHook)

declare -a pkgHookVarVars=(pkgBuildHookVars pkgHostHookVars pkgTargetHookVars)

# those variables are declared here, since where and if they are used varies
declare -a PRE_FIX_HOOKS FIX_OUTPUT_HOOKS PRE_CONFIGURE_HOOKS POST_FIX_HOOKS POST_UNPACK_HOOKS UNPACK_CMD_HOOKS

# Add env hooks for all sorts of deps with the specified host offset.
addEnvHooks() {
    local depHostOffset="$1"
    shift
    local pkgHookVarsSlice="${pkgHookVarVars[$depHostOffset + 1]}[@]"
    local pkgHookVar
    for pkgHookVar in "${!pkgHookVarsSlice}"; do
        eval "${pkgHookVar}s"'+=("$@")'
    done
}


# Propagated dep files

declare -a propagatedBuildDepFiles=(
    propagated-build-build-deps
    propagated-native-build-inputs # Legacy name for back-compat
    propagated-build-target-deps
)
declare -a propagatedHostDepFiles=(
    propagated-host-host-deps
    propagated-build-inputs # Legacy name for back-compat
)
declare -a propagatedTargetDepFiles=(
    propagated-target-target-deps
)
declare -a propagatedDepFilesVars=(
    propagatedBuildDepFiles
    propagatedHostDepFiles
    propagatedTargetDepFiles
)

# Platform offsets: build = -1, host = 0, target = 1
declare -a allPlatOffsets=(-1 0 1)


# Mutually-recursively find all build inputs. See the dependency section of the
# stdenv chapter of the Nixpkgs manual for the specification this algorithm
# implements.
findInputs() {
    local -r pkg="$1"
    local -r hostOffset="$2"
    local -r targetOffset="$3"

    # Sanity check
    (( hostOffset <= targetOffset )) || exit 1

    # shellcheck disable=SC1087
    local varVar="${pkgAccumVarVars[hostOffset + 1]}"
    # shellcheck disable=SC1087
    local varRef="$varVar[$((targetOffset - hostOffset))]"
    local var="${!varRef}"
    unset -v varVar varRef

    # shellcheck disable=SC1087
    local varSlice="$var[*]"
    # ${..-} to hack around old bash empty array problem
    case " ${!varSlice-} " in
        *" $pkg "*) return 0 ;;
    esac
    unset -v varSlice

    eval "$var"'+=("$pkg")'

    if ! [ -e "$pkg" ]; then
        echo "build input $pkg does not exist" >&2
        exit 1
    fi

    # The current package's host and target offset together
    # provide a <=-preserving homomorphism from the relative
    # offsets to current offset
    function mapOffset() {
        local -r inputOffset="$1"
        local -n outputOffset="$2"
        if (( inputOffset <= 0 )); then
            outputOffset=$((inputOffset + hostOffset))
        else
            outputOffset=$((inputOffset - 1 + targetOffset))
        fi
    }

    # Host offset relative to that of the package whose immediate
    # dependencies we are currently exploring.
    local relHostOffset
    for relHostOffset in "${allPlatOffsets[@]}"; do
        # `+ 1` so we start at 0 for valid index
        local files="${propagatedDepFilesVars[relHostOffset + 1]}"

        # Host offset relative to the package currently being
        # built---as absolute an offset as will be used.
        local hostOffsetNext
        mapOffset "$relHostOffset" hostOffsetNext

        # Ensure we're in bounds relative to the package currently
        # being built.
        (( -1 <= hostOffsetNext && hostOffsetNext <= 1 )) || continue

        # Target offset relative to the *host* offset of the package
        # whose immediate dependencies we are currently exploring.
        local relTargetOffset
        for relTargetOffset in "${allPlatOffsets[@]}"; do
            (( "$relHostOffset" <= "$relTargetOffset" )) || continue

            local fileRef="${files}[$relTargetOffset - $relHostOffset]"
            local file="${!fileRef}"
            unset -v fileRef

            # Target offset relative to the package currently being
            # built.
            local targetOffsetNext
            mapOffset "$relTargetOffset" targetOffsetNext

            # Once again, ensure we're in bounds relative to the
            # package currently being built.
            (( -1 <= hostOffsetNext && hostOffsetNext <= 1 )) || continue

            [[ -f "$pkg/nix-support/$file" ]] || continue

            local pkgNext
            read -r -d '' pkgNext < "$pkg/nix-support/$file" || true
            for pkgNext in $pkgNext; do
                findInputs "$pkgNext" "$hostOffsetNext" "$targetOffsetNext"
            done
        done
    done
}

# The way we handle deps* and *Inputs works with structured attrs
# either enabled or disabled.  For this it's convenient that the items
# in each list must be store paths, and therefore space-free.

# Make sure all are at least defined as empty
: "${DEPS_BUILD_BUILD=}" "${PROPAGATED_BUILD_BUILD=}"
: "${DEPS_BUILD_HOST=}" "${PROPAGATED_BUILD_HOST=}" "${default_build_host=}"
: "${DEPS_BUILD_TARGET=}" "${PROPAGATED_BUILD_TARGET=}"
: "${DEPS_HOST_HOST=}" "${PROPAGATED_HOST_HOST=}"
: "${DEPS_HOST_TARGET=}" "${PROPAGATED_HOST_TARGET=}" "${default_host_target=}"
: "${DEPS_TARGET_TARGET=}" "${PROPAGATED_TARGET_TARGET=}"

for pkg in "${DEPS_BUILD_BUILD[@]}" "${PROPAGATED_BUILD_BUILD[@]}"; do
    findInputs "$pkg" -1 -1
done
for pkg in "${DEPS_BUILD_HOST[@]}" "${PROPAGATED_BUILD_HOST[@]}"; do
    findInputs "$pkg" -1  0
done
for pkg in "${DEPS_BUILD_TARGET[@]}" "${PROPAGATED_BUILD_TARGET[@]}"; do
    findInputs "$pkg" -1  1
done
for pkg in "${DEPS_HOST_HOST[@]}" "${PROPAGATED_HOST_HOST[@]}"; do
    findInputs "$pkg"  0  0
done
for pkg in "${HOST_TARGET[@]}" "${PROPAGATED_HOST_TARGET[@]}" ; do
    findInputs "$pkg"  0  1
done
for pkg in "${DEPS_TARGET_TARGET[@]}" "${PROPAGATED_TARGET_TARGET[@]}"; do
    findInputs "$pkg"  1  1
done
# Default inputs must be processed last
for pkg in "${default_build_host[@]}"; do
    findInputs "$pkg" -1  0
done
for pkg in "${default_host_target[@]}"; do
    findInputs "$pkg"  0  1
done

# Add package to the future PATH and run setup hooks
activatePackage() {
    local pkg="$1"
    local -r hostOffset="$2"
    local -r targetOffset="$3"

    # Sanity checkSC1091
    (( hostOffset <= targetOffset )) || exit 1

    if [ -f "$pkg" ]; then
        source "$pkg"
    fi

    # Only dependencies whose host platform is guaranteed to match the
    # build platform are included here. That would be `depsBuild*`,
    # and legacy `DEPS_BUILD_HOST`, in general. If we aren't cross
    # compiling, however, everything can be put on the PATH. To ease
    # the transition, we do include everything in that case.
    #
    # TODO(@Ericson2314): Don't special-case native compilation
    if [[ -z "${strictDeps-}" || "$hostOffset" -le -1 ]]; then
        addToSearchPath _PATH "$pkg/bin"
    fi

    if (( hostOffset <= -1 )); then
        addToSearchPath _XDG_DATA_DIRS "$pkg/share"
    fi

    if [[ "$hostOffset" -eq 0 && -d "$pkg/bin" ]]; then
        addToSearchPath _HOST_PATH "$pkg/bin"
    fi

    if [[ -f "$pkg/nix-support/setup-hook" ]]; then
        source "$pkg/nix-support/setup-hook"
    fi
}

_activatePkgs() {
    local hostOffset targetOffset
    local pkg

    for hostOffset in "${allPlatOffsets[@]}"; do
        local pkgsVar="${pkgAccumVarVars[hostOffset + 1]}"
        for targetOffset in "${allPlatOffsets[@]}"; do
            (( hostOffset <= targetOffset )) || continue
            local pkgsRef="${pkgsVar}[$targetOffset - $hostOffset]"
            local pkgsSlice="${!pkgsRef}[@]"
            for pkg in ${!pkgsSlice+"${!pkgsSlice}"}; do
                activatePackage "$pkg" "$hostOffset" "$targetOffset"
            done
        done
    done
}

# Run the package setup hooks and build _PATH
_activatePkgs

# Set the relevant environment variables to point to the build inputs
# found above.
#
# These `depOffset`s, beyond indexing the arrays, also tell the env
# hook what sort of dependency (ignoring propagatedness) is being
# passed to the env hook. In a real language, we'd append a closure
# with this information to the relevant env hook array, but bash
# doesn't have closures, so it's easier to just pass this in.
_addToEnv() {
    local depHostOffset depTargetOffset
    local pkg

    for depHostOffset in "${allPlatOffsets[@]}"; do
        local hookVar="${pkgHookVarVars[depHostOffset + 1]}"
        local pkgsVar="${pkgAccumVarVars[depHostOffset + 1]}"
        for depTargetOffset in "${allPlatOffsets[@]}"; do
            (( depHostOffset <= depTargetOffset )) || continue
            local hookRef="${hookVar}[$depTargetOffset - $depHostOffset]"
            if [[ -z "${strictDeps-}" ]]; then

                # Keep track of which packages we have visited before.
                local visitedPkgs=""

                # Apply environment hooks to all packages during native
                # compilation to ease the transition.
                #
                # TODO(@Ericson2314): Don't special-case native compilation
                for pkg in \
                    "${pkgsBuildBuild[@]}" \
                    "${pkgsBuildHost[@]}" \
                    "${pkgsBuildTarget[@]}" \
                    "${pkgsHostHost[@]}" \
                    "${pkgsHostTarget[@]}" \
                    "${pkgsTargetTarget[@]}"
                do
                    if [[ "$visitedPkgs" = *"$pkg"* ]]; then
                        continue
                    fi
                    run_hook "${!hookRef}" "$pkg"
                    visitedPkgs+=" $pkg"
                done
            else
                local pkgsRef="${pkgsVar}[$depTargetOffset - $depHostOffset]"
                local pkgsSlice="${!pkgsRef}[@]"
                for pkg in ${!pkgsSlice+"${!pkgsSlice}"}; do
                    run_hook "${!hookRef}" "$pkg"
                done
            fi
        done
    done
}

# Run the package-specific hooks set by the setup-hook scripts.
_addToEnv


# Unset setup-specific declared variables
unset allPlatOffsets
unset pkgBuildAccumVars pkgHostAccumVars pkgTargetAccumVars pkgAccumVarVars
unset pkgBuildHookVars pkgHostHookVars pkgTargetHookVars pkgHookVarVars
unset propagatedDepFilesVars


_add_rpath_prefix "$out"


# Set the TZ (timezone) environment variable, otherwise commands like
# `date' will complain (e.g., `Tue Mar 9 10:01:47 Local time zone must
# be set--see zic manual page 2004').
export TZ=UTC


# Set the prefix.  This is generally $out, but it can be overriden,
# for instance if we just want to perform a test build/install to a
# temporary location and write a build report to $out.
if [ -z "${prefix:-}" ]; then
    prefix="$out";
fi

PATH="${_PATH-}${_PATH:+${PATH:+:}}$PATH"
HOST_PATH="${_HOST_PATH-}${_HOST_PATH:+${HOST_PATH:+:}}$HOST_PATH"
export XDG_DATA_DIRS="${_XDG_DATA_DIRS-}${_XDG_DATA_DIRS:+${XDG_DATA_DIRS:+:}}${XDG_DATA_DIRS-}"

echo "final path: $PATH"
echo "final host path: $HOST_PATH"
echo "final data dirs: $XDG_DATA_DIRS"

unset _PATH
unset _HOST_PATH
unset _XDG_DATA_DIRS


OXIDE_BUILD_CORES="${OXIDE_BUILD_CORES:-1}"
if ((OXIDE_BUILD_CORES <= 0)); then
  guess=$(nproc 2>/dev/null || true)
  ((OXIDE_BUILD_CORES = guess <= 0 ? 1 : guess))
fi
export OXIDE_BUILD_CORES


######################################################################
# Textual substitution functions.

_substituteStream_has_warned_replace_deprecation=false
substituteStream() {
    local var=$1
    local description=$2
    shift 2

    while (( "$#" )); do
        local replace_mode="$1"
        case "$1" in
            --replace)
                # deprecated 2023-11-22
                # this will either get removed, or switch to the behaviour of --replace-fail in the future
                if ! "$_substituteStream_has_warned_replace_deprecation"; then
                    echo "substituteStream() in derivation $name: WARNING: '--replace' is deprecated, use --replace-{fail,warn,quiet}. ($description)" >&2
                    _substituteStream_has_warned_replace_deprecation=true
                fi
                replace_mode='--replace-warn'
                ;&
            --replace-quiet|--replace-warn|--replace-fail)
                pattern="$2"
                replacement="$3"
                shift 3
                if ! [[ "${!var}" == *"$pattern"* ]]; then
                    if [ "$replace_mode" == --replace-warn ]; then
                        printf "substituteStream() in derivation $name: WARNING: pattern %q doesn't match anything in %s\n" "$pattern" "$description" >&2
                    elif [ "$replace_mode" == --replace-fail ]; then
                        printf "substituteStream() in derivation $name: ERROR: pattern %q doesn't match anything in %s\n" "$pattern" "$description" >&2
                        return 1
                    fi
                fi
                eval "$var"'=${'"$var"'//"$pattern"/"$replacement"}'
                ;;

            --subst-var)
                local varName="$2"
                shift 2
                # check if the used nix attribute name is a valid bash name
                if ! [[ "$varName" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
                    echo "substituteStream() in derivation $name: ERROR: substitution variables must be valid Bash names, \"$varName\" isn't." >&2
                    return 1
                fi
                if [ -z ${!varName+x} ]; then
                    echo "substituteStream() in derivation $name: ERROR: variable \$$varName is unset" >&2
                    return 1
                fi
                pattern="@$varName@"
                replacement="${!varName}"
                eval "$var"'=${'"$var"'//"$pattern"/"$replacement"}'
                ;;

            --subst-var-by)
                pattern="@$2@"
                replacement="$3"
                eval "$var"'=${'"$var"'//"$pattern"/"$replacement"}'
                shift 3
                ;;

            *)
                echo "substituteStream() in derivation $name: ERROR: Invalid command line argument: $1" >&2
                return 1
                ;;
        esac
    done

    printf "%s" "${!var}"
}

# put the content of a file in a variable
# fail loudly if provided with a binary (containing null bytes)
consumeEntire() {
    # read returns non-0 on EOF, so we want read to fail
    if IFS='' read -r -d '' "$1" ; then
        echo "consumeEntire(): ERROR: Input null bytes, won't process" >&2
        return 1
    fi
}

substitute() {
    local input="$1"
    local output="$2"
    shift 2

    if [ ! -f "$input" ]; then
        echo "substitute(): ERROR: file '$input' does not exist" >&2
        return 1
    fi

    consumeEntire content < "$input"

    if [ -e "$output" ]; then chmod +w "$output"; fi
    substituteStream content "file '$input'" "$@" > "$output"
}

substituteInPlace() {
    local -a fileNames=()
    for arg in "$@"; do
        if [[ "$arg" = "--"* ]]; then
            break
        fi
        fileNames+=("$arg")
        shift
    done
    if ! [[ "${#fileNames[@]}" -gt 0 ]]; then
        echo >&2 "substituteInPlace called without any files to operate on (files must come before options!)"
        return 1
    fi

    for file in "${fileNames[@]}"; do
        substitute "$file" "$file" "$@"
    done
}

_allFlags() {
    # Export some local variables for the `awk` below so some substitutions (such as name)
    # don't have to be in the env attrset when `__structuredAttrs` is enabled.
    export system pname name version
    while IFS='' read -r varName; do
        args+=("--subst-var" "$varName")
    done < <(awk 'BEGIN { for (v in ENVIRON) if (v ~ /^[a-z][a-zA-Z0-9_]*$/) print v }')
}

substituteAllStream() {
    local -a args=()
    _allFlags

    substituteStream "$1" "$2" "${args[@]}"
}

# Substitute all environment variables that start with a lowercase character and
# are valid Bash names.
substituteAll() {
    local input="$1"
    local output="$2"

    local -a args=()
    _allFlags

    substitute "$input" "$output" "${args[@]}"
}


substituteAllInPlace() {
    local fileName="$1"
    shift
    substituteAll "$fileName" "$fileName" "$@"
}


######################################################################
# What follows is the generic builder.

# Utility function: echo the base name of the given path, with the
# prefix `HASH-' removed, if present.
strip_hash() {
    local stripped_name case_match_opt=0
    # On separate line for `set -e`
    stripped_name="$(basename -- "$1")"
    shopt -q nocasematch && case_match_opt=1
    shopt -u nocasematch
    if [[ "$stripped_name" =~ ^[a-z0-9]{64}- ]]; then
        echo "${stripped_name:65}"
    else
        echo "$stripped_name"
    fi
    if (( case_match_opt )); then shopt -s nocasematch; fi
}


UNPACK_CMD_HOOKS+=(_default_unpack)
_default_unpack() {
    local fn="$1"
    local destination

    if [ -d "$fn" ]; then

        destination="$(strip_hash "$fn")"

        if [ -e "$destination" ]; then
            echo "Cannot copy $fn to $destination: destination already exists!"
            echo "Did you specify two \"srcs\" with the same \"name\"?"
            return 1
        fi

        # We can't preserve hardlinks because they may have been
        # introduced by store optimization, which might break things
        # in the build.
        cp -r --preserve=timestamps --reflink=auto -- "$fn" "$destination"

    else

        case "$fn" in
            *.tar.xz | *.tar.lzma | *.txz)
                # Don't rely on tar knowing about .xz.
                # Additionally, we have multiple different xz binaries with different feature sets in different
                # stages. The XZ_OPT env var is only used by the full "XZ utils" implementation, which supports
                # the --threads (-T) flag. This allows us to enable multithreaded decompression exclusively on
                # that implementation, without the use of complex bash conditionals and checks.
                # Since tar does not control the decompression, we need to
                # disregard the error code from the xz invocation. Otherwise,
                # it can happen that tar exits earlier, causing xz to fail
                # from a SIGPIPE.
                (XZ_OPT="--threads=$OXIDE_BUILD_CORES" xz -d < "$fn"; true) | tar xf - --mode=+w --warning=no-timestamp
                ;;
            *.tar | *.tar.* | *.tgz | *.tbz2 | *.tbz)
                # GNU tar can automatically select the decompression method
                # (info "(tar) gzip").
                tar xf "$fn" --mode=+w --warning=no-timestamp
                ;;
            *)
                return 1
                ;;
        esac

    fi
}


unpack_file() {
    cur_src="$1"
    echo "unpacking source archive $cur_src"
    if ! run_one_hook UNPACK_CMD "$cur_src"; then
        echo "do not know how to unpack source archive $cur_src"
        exit 1
    fi
}


unpack_phase() {
    run_hook PRE_UNPACK

    if [ -z "${SRCS:-}" ]; then
        if [ -z "${SRC:-}" ]; then
            # shellcheck disable=SC2016
            echo 'variable $SRC or $SRCS should point to the source'
            exit 1
        fi
        SRCS="$SRC"
    fi

    local -a srcs_array
    concatTo srcs_array SRCS

    # To determine the source directory created by unpacking the
    # source archives, we record the contents of the current
    # directory, then look below which directory got added.  Yeah,
    # it's rather hacky.
    local dirs_before=""
    for i in *; do
        if [ -d "$i" ]; then
            dirs_before="$dirs_before $i "
        fi
    done

    # Unpack all source archives.
    for i in "${srcs_array[@]}"; do
        unpack_file "$i"
    done

    # Find the source directory.

    # set to empty if unset
    : "${SRC_ROOT=}"

    if [ -z "$SRC_ROOT" ]; then
        for i in *; do
            if [ -d "$i" ]; then
                case $dirs_before in
                    *\ $i\ *)
                        ;;
                    *)
                        if [ -n "$SRC_ROOT" ]; then
                            echo "unpacker produced multiple directories"
                            exit 1
                        fi
                        SRC_ROOT="$i"
                        ;;
                esac
            fi
        done
    fi

    if [ -z "$SRC_ROOT" ]; then
        echo "unpacker appears to have produced no directories"
        exit 1
    fi

    echo "source root is $SRC_ROOT"

    # By default, add write permission to the sources.  This is often
    # necessary when sources have been copied from other store
    # locations.
    chmod -R u+w -- "$SRC_ROOT"

    run_hook POST_UNPACK
}


patch_phase() {
    run_hook PRE_PATCH

    local -a patches_array
    concatTo patches_array patches

    for i in "${patches_array[@]}"; do
        echo "applying patch $i"
        local uncompress=cat
        case "$i" in
            *.gz)
                uncompress="gzip -d"
                ;;
            *.bz2)
                uncompress="bzip2 -d"
                ;;
            *.xz)
                uncompress="xz -d"
                ;;
            *.lzma)
                uncompress="lzma -d"
                ;;
        esac

        local -a flags_array
        concatTo flags_array PATCH_FLAGS=-p1
        # "2>&1" is a hack to make patch fail if the decompressor fails (nonexistent patch, etc.)
        # shellcheck disable=SC2086
        $uncompress < "$i" 2>&1 | patch "${flags_array[@]}"
    done

    run_hook POST_PATCH
}


fix_lib_tool() {
    local search_path
    for flag in $OXIDE_LDFLAGS; do
        case $flag in
            -L*)
                search_path+=" ${flag#-L}"
                ;;
        esac
    done

    sed -i "$1" \
        -e "s^eval \(sys_lib_search_path=\).*^\1'${search_path:-}'^" \
        -e 's^eval sys_lib_.+search_path=.*^^'
}


configure_phase() {
    run_hook PRE_CONFIGURE

    # set to empty if unset
    : "${CONFIGURE_SCRIPT=}"

    if [[ -z "$CONFIGURE_SCRIPT" && -x ./configure ]]; then
        CONFIGURE_SCRIPT=./configure
    fi

    if [ -z "${DONT_FIX_LIBTOOL:-}" ]; then
        export lt_cv_deplibs_check_method="${lt_cv_deplibs_check_method-pass_all}"
        local i
        find . -iname "ltmain.sh" -print0 | while IFS='' read -r -d '' i; do
            echo "fixing libtool script $i"
            fix_lib_tool "$i"
        done

        # replace `/usr/bin/file` with `file` in any `configure`
        # scripts with vendored libtool code.  Preserve mtimes to
        # prevent some packages (e.g. libidn2) from spontaneously
        # autoreconf'ing themselves
        CONFIGURE_MTIME_REFERENCE=$(mktemp configure.mtime.reference.XXXXXX)
        find . \
          -executable \
          -type f \
          -name configure \
          -exec grep -l 'GNU Libtool is free software; you can redistribute it and/or modify' {} \; \
          -exec touch -r {} "$CONFIGURE_MTIME_REFERENCE" \; \
          -exec sed -i s_/usr/bin/file_file_g {} \;    \
          -exec touch -r "$CONFIGURE_MTIME_REFERENCE" {} \;
        rm -f "$CONFIGURE_MTIME_REFERENCE"
    fi

    if [[ -z "${DONT_ADD_PREFIX:-}" && -n "$prefix" ]]; then
        prependToVar CONFIGURE_FLAGS "${prefixKey:---prefix=}$prefix"
    fi

    if [[ -f "$CONFIGURE_SCRIPT" ]]; then
        # Add --disable-dependency-tracking to speed up some builds.
        if [ -z "${DONT_ADD_DISABLED_EPTRACK:-}" ]; then
            if grep -q dependency-tracking "$CONFIGURE_SCRIPT"; then
                prependToVar CONFIGURE_FLAGS --disable-dependency-tracking
            fi
        fi

        # By default, disable static builds.
        if [ -z "${DONT_DISABLE_STATIC:-}" ]; then
            if grep -q enable-static "$CONFIGURE_SCRIPT"; then
                prependToVar CONFIGURE_FLAGS --disable-static
            fi
        fi

        if [ -z "${dontPatchShebangsInConfigure:-}" ]; then
            # TODO:
            # patchShebangs --build "$CONFIGURE_SCRIPT"
            echo "TODO: patchShebangs"
        fi
    fi

    if [ -n "$CONFIGURE_SCRIPT" ]; then
        local -a flags_array
        concatTo flags_array CONFIGURE_FLAGS

        # shellcheck disable=SC2086
        $CONFIGURE_SCRIPT "${flags_array[@]}"
        unset flags_array
    else
        echo "no configure script, doing nothing"
    fi

    run_hook POST_CONFIGURE
}


build_phase() {
    run_hook PRE_BUILD

    if [[ -z "${MAKE_FLAGS-}" && -z "${MAKEFILE:-}" && ! ( -e Makefile || -e makefile || -e GNUmakefile ) ]]; then
        echo "no Makefile or custom buildPhase, doing nothing"
    else
        found_make_file=1

        # shellcheck disable=SC2086
        local flags_array=(
            ${ENABLE_PARALLEL_BUILDING:+-j${OXIDE_BUILD_CORES}}
            SHELL="$SHELL"
        )
        concatTo flags_array MAKE_FLAGS BUILD_FLAGS

        make ${MAKEFILE:+-f $MAKEFILE} "${flags_array[@]}"
        unset flags_array
    fi

    run_hook POST_BUILD
}


check_phase() {
    run_hook PRE_CHECK

    if [[ -z "${found_make_file:-}" ]]; then
        echo "no Makefile or custom check_phase, doing nothing"
        run_hook postCheck
        return
    fi

    if [[ -z "${CHECK_TARGET:-}" ]]; then
        #TODO(@oxij): should flags_array influence make -n?
        if make -n ${MAKEFILE:+-f $MAKEFILE} check >/dev/null 2>&1; then
            CHECK_TARGET="check"
        elif make -n ${MAKEFILE:+-f $MAKEFILE} test >/dev/null 2>&1; then
            CHECK_TARGET="test"
        fi
    fi

    if [[ -z "${CHECK_TARGET:-}" ]]; then
        echo "no check/test target in ${MAKEFILE:-Makefile}, doing nothing"
    else
        # Old bash empty array hack
        # shellcheck disable=SC2086
        local flags_array=(
            ${ENABLE_PARALLEL_CHECKING:+-j${OXIDE_BUILD_CORES}}
            SHELL="$SHELL"
        )

        concatTo flags_array MAKE_FLAGS CHECK_FLAGS=VERBOSE=y CHECK_TARGET

        make ${MAKEFILE:+-f $MAKEFILE} "${flags_array[@]}"

        unset flags_array
    fi

    run_hook POST_CHECK
}


install_phase() {
    run_hook PRE_INSTALL

    # Dont reuse 'found_make_file' set in buildPhase, a makefile may have been created in buildPhase
    if [[ -z "${makeFlags-}" && -z "${makefile:-}" && ! ( -e Makefile || -e makefile || -e GNUmakefile ) ]]; then
        echo "no Makefile or custom installPhase, doing nothing"
        run_hook POST_INSTALL
        return
    else
        found_make_file=1
    fi

    if [ -n "$prefix" ]; then
        mkdir -p "$prefix"
    fi

    # shellcheck disable=SC2086
    local flags_array=(
        ${ENABLE_PARALLEL_INSTALLING:+-j${OXIDE_BUILD_CORES}}
        SHELL="$SHELL"
    )

    concatTo flags_array MAKE_FLAGS INSTALL_FLAGS INSTALL_TARGETS=install

    make ${MAKEFILE:+-f $MAKEFILE} "${flags_array[@]}"
    unset flags_array

    run_hook POST_INSTALL
}


# The fix phase performs generic, package-independent stuff, like
# stripping binaries, running patchelf and setting
# propagated-build-inputs.
fix_phase() {
    # Make sure everything is writable so "strip" et al. work.
    local output
    for output in $(get_all_output_names); do
        # for set*id bits see #300635
        if [ -e "${!output}" ]; then chmod -R u+w,u-s,g-s "${!output}"; fi
    done

    run_hook PRE_FIX

    # Apply fix to each output.
    local output
    for output in $(get_all_output_names); do
        prefix="${!output}" run_hook fixupOutput
    done

    run_hook POST_FIX
}


install_check_phase() {
    run_hook PRE_INSTALL_CHECK

    if [[ -z "${found_make_file:-}" ]]; then
        echo "no Makefile or custom install_check_phase, doing nothing"
    elif [[ -z "${INSTALL_CHECK_TARGET:-}" ]] \
       && ! make -n ${MAKEFILE:+-f $MAKEFILE} "${INSTALL_CHECK_TARGET:-installcheck}" >/dev/null 2>&1; then
        echo "no installcheck target in ${MAKEFILE:-Makefile}, doing nothing"
    else
        # Old bash empty array hack
        # shellcheck disable=SC2086
        local flags_array=(
            ${ENABLE_PARALLEL_CHECKING:+-j${OXIDE_BUILD_CORES}}
            SHELL="$SHELL"
        )

        concatTo flags_array MAKE_FLAGS INSTALL_CHECK_FLAGS INSTALL_CHECK_TARGET=installcheck

        make ${MAKEFILE:+-f $MAKEFILE} "${flags_array[@]}"
        unset flags_array
    fi

    run_hook POST_INSTALL_CHECK
}

run_phase() {
    local phase="$*"
    local phase_fn
    phase_fn=$(echo "$phase" | awk '{print tolower($0)}')
    if [[ "$phase" = UNPACK_PHASE && -z "${UNPACK:-}" ]]; then return; fi
    if [[ "$phase" = PATCH_PHASE && -z "${PATCH:-}" ]]; then return; fi
    if [[ "$phase" = CONFIGURE_PHASE && -z "${CONFIGURE:-}" ]]; then return; fi
    if [[ "$phase" = BUILD_PHASE && -z "${BUILD:-}" ]]; then return; fi
    if [[ "$phase" = CHECK_PHASE && -z "${CHECK:-}" ]]; then return; fi
    if [[ "$phase" = INSTALL_PHASE && -z "${INSTALL:-}" ]]; then return; fi
    if [[ "$phase" = FIX_PHASE && -z "${FIX:-}" ]]; then return; fi
    if [[ "$phase" = INSTALL_CHECK_PHASE && -z "${INSTALL_CHECK:-}" ]]; then return; fi

    echo "Running phase: $phase"

    # Evaluate the variable named $phase if it exists, otherwise the
    # function named $phase_fn.
    eval "${!phase:-$phase_fn}"

    echo "$phase completed"

    if [ "$phase" = UNPACK_PHASE ]; then
        # make sure we can cd into the directory
        [ -n "${SRC_ROOT:-}" ] && chmod +x -- "${SRC_ROOT}"

        cd -- "${SRC_ROOT:-.}"
    fi
}


generic_build() {
    # variable used by our gzip wrapper to add -n.
    # gzip is in common-path.nix and is added to nix-shell but we only want to change its behaviour in nix builds. do not move to a setupHook in gzip.
    export GZIP_NO_TIMESTAMPS=1

    if [ -n "${BUILD_COMMAND:-}" ]; then
        eval "$BUILD_COMMAND"
        return
    fi

    if [ -z "${phases[*]:-}" ]; then
        phases="${PRE_PHASES[*]:-} UNPACK_PHASE PATCH_PHASE ${PRE_CONFIGURE_PHASES[*]:-} \
            CONFIGURE_PHASE ${PRE_BUILD_PHASES[*]:-} BUILD_PHASE CHECK_PHASE \
            ${PRE_INSTALL_PHASES[*]:-} INSTALL_PHASE ${PRE_FIX_PHASES[*]:-} FIX_PHASE INSTALL_CHECK_PHASE \
            ${POST_PHASES[*]:-}";
    fi

    # This relies on phase name space-free, which it must be because it's the name
    # of either a shell variable or a shell function.
    for phase in ${phases}; do
        run_phase "$phase"
    done
}

# Execute the post-hooks.
run_hook POST_HOOK 
