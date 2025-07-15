# shellcheck disable=SC2148,SC2154

export PATH=
for i in $INITIAL_PATH; do
    if [ "$i" = / ]; then i=; fi
    PATH=$PATH${PATH:+:}$i/bin
done

mkdir "$out"

{
  echo "export SHELL=$SHELL"
  echo "initial_path=\"$INITIAL_PATH\""
  echo "default_build_host=\"$DEFAULT_BUILD_HOST\""
  echo "default_host_target=\"$DEFAULT_HOST_TARGET\""
  echo "$PRE_HOOK"
  cat "$SETUP"
} > "$out/setup"

