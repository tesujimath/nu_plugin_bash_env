export def main [
  path?: string
  --export: list
  --shellvars (-s)
  --fn (-f): list
] {
  let fn_args = if ($fn | is-not-empty) {
    ['--shellfns' ($fn | str join ',')]
  } else {
    []
  }

  let path_args = if $path != null {
    [($path | path expand)]
  } else {
    []
  }

  # print -e bash-env-json ...($fn_args ++ $path_args)
  let raw = bash-env-json ...($fn_args ++ $path_args) | from json

  let error = $raw | get -i error
  if $error != null {
    error make { msg: $error }
  }

  if ($export | is-not-empty) {
    print "warning: --export is deprecated, use --shellvars(-s) instead"
    let exported_shellvars = ($raw.shellvars | select -i ...$export)
    $raw.env | merge ($exported_shellvars)
  } else if $shellvars or ($fn | is-not-empty) {
    $raw
  } else {
    $raw.env
  }
}
