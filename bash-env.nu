export def main [
  path?: string
  --export: list
] {
  let raw = if $path != null {
    bash-env-json ($path | path expand) | from json
  } else {
    bash-env-json | from json
  }

  if ($export | is-not-empty) {
    let exported_shellvars = ($raw.shellvars | select -i ...$export)
    $raw.env | merge ($exported_shellvars)
  } else {
    $raw.env
  }
}
