use std assert

# TODO use testing.nu testing module,
# which wasn't working at the time I wrote these tests

#[test]
def test_shell_variables [] {
  let actual = (echo "A=123" | bash-env -s).shellvars
  let expected = { A: "123" }
  assert equal $actual $expected
}

#[test]
def test_shell_variables_from_file [] {
  let actual = bash-env -s tests/shell-variables.env
  let expected = { shellvars: { A: "not exported" } env: { B: "exported" } }
  assert equal $actual $expected
}

#[test]
def test_shell_functions [] {
  let actual = bash-env -f [f2 f3] tests/shell-functions.env
  let expected = {
    "env": {
      "B": "1",
      "A": "1"
    },
    "shellvars": {},
    "fn": {
      "f2": {
        "env": {
          "B": "2",
          "A": "2"
        },
        "shellvars": {
          "C2": "I am shell variable C2"
        }
      },
      "f3": {
        "env": {
          "B": "3",
          "A": "3"
        },
        "shellvars": {
          "C3": "I am shell variable C3"
        }
      }
    }
  }
  assert equal $actual $expected
}

export def main [] {
  test_shell_variables
  test_shell_variables_from_file
  test_shell_functions

  print "All tests passed"
}
