use std assert

# TODO use testing.nu testing module,
# which wasn't working at the time I wrote these tests

#[test]
def test_echo [] {
  let actual = echo "export A=123" | bash-env
  let expected = { A: "123" }
  assert equal $actual $expected
}

#[test]
def test_not_exported [] {
  let actual = echo "A=123" | bash-env
  let expected = { }
  assert equal $actual $expected
}

#[test]
def test_export_shell_variables [] {
  let actual = echo "A=123" | bash-env --export [A]
  let expected = { A: "123" }
  assert equal $actual $expected
}

#[test]
def test_shell_variables_from_file [] {
  let actual = bash-env tests/shell-variables.env
  let expected = { B: "exported" }
  assert equal $actual $expected
}

#[test]
def test_export_shell_variables_from_file [] {
  let actual = bash-env --export [A] tests/shell-variables.env
  let expected = { A: "not exported" B: "exported" }
  assert equal $actual $expected
}

#[test]
def test_empty_value [] {
  let actual = echo "export A=\"\"" | bash-env
  let expected = { A: "" }
  assert equal $actual $expected
}

#[test]
def test_simple_file [] {
  let actual = bash-env tests/simple.env
  let expected = { A: "a" B: "b" }
  assert equal $actual $expected
}

#[test]
def test_cat_simple_file [] {
  let actual = cat tests/simple.env | bash-env
  let expected = { A: "a" B: "b" }
  assert equal $actual $expected
}

#[test]
def test_nasty_values_from_file [] {
  let actual = bash-env "tests/Ming's menu of (merciless) monstrosities.env"
  let expected = {
    SPACEMAN: "One small step for a man ..."
    QUOTE: "\"Well done!\" is better than \"Well said!\""
    MIXED_BAG: "Did the sixth sheik's sixth sheep say \"baa\", or not?"
  }
  assert equal $actual $expected
}

export def run_bash_env_tests [] {
  test_echo
  test_not_exported
  test_export_shell_variables
  test_shell_variables_from_file
  test_export_shell_variables_from_file
  test_empty_value
  test_simple_file
  test_cat_simple_file
  test_nasty_values_from_file

  print "All tests passed"
}
