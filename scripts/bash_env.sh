#!/usr/bin/env bash
#
# Copyright 2023-24 Simon Guest
#
# Permission is hereby granted, free of charge, to any person
# obtaining a copy of this software and associated documentation files
# (the “Software”), to deal in the Software without restriction,
# including without limitation the rights to use, copy, modify, merge,
# publish, distribute, subl# icense, and/or sell copies of the
# Software, and to permit persons to whom the Software is furnished to
# do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be
# included in all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
# BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
# ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
# CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

shopt -s extglob

function send_value() {
    # jq -R produces nothing on empty input, but we want ""
    if test -n "$1"; then
        echo -n "$1" | jq -R
    else
        echo -n '""'
    fi
}

function send_environment() {
    local _n_env _head _tail _name _call_id
    declare -a _tail
    _call_id="$1"
    shift

    _n_env=$#

    _head="$1"
    shift
    _tail=("$@")

    # header
    echo -n "{\"Env\":["

    # names and values
    if test $_n_env -gt 0; then
        echo -n "{\"k\":\"$_head\",\"v\":"
        send_value "${!_head}"
        echo -n "}"

        for _name in "${_tail[@]}"; do
            if test -v "$_name"; then
                echo -n ",{\"k\":\"$_name\",\"v\":"
                send_value "${!_name}"
                echo -n "}"
            else
                # unset, TODO
                :
            fi
        done
    fi

    # trailer
    echo ']}'
}

function send_error() {
    local _msg
    _msg="$1"
    jq -c <<EOF
{
  "Error": "$_msg"
}
EOF
}

function eval_and_or_source_then_send_environment() {
    local _source _path _name _value _env_old _env_new _env_changed_or_unset _call_id _exports _export
    _source="$1"
    shift

    _path="$1"
    shift

    # final argument is array of exports, if any
    declare -a _exports
    _exports=("$@")

    # get previous env
    declare -A _env_old
    while IFS='=' read -r -d '' _name _value; do
        _env_old[$_name]="${_value@Q}"
    done < <(env -0)

    # source from file
    if test -n "$_path"; then

        if test ! -r "$_path"; then
            send_error "no such file '$_path'"
            return
        fi

        # ShellCheck can't cope with sourcing from an unknown path
        # shellcheck disable=SC1090
        if ! source "$_path" >&2; then
            send_error "failed to load environment from '$_path'"
            return 1
        fi
    fi

    # eval from _source
    if ! eval "$_source" >&2; then
        send_error "failed to load environment from stdin"
        return 1
    fi

    # export shell variables to environment as specified, if any
    for _export in "${_exports[@]}"; do
        export "${_export}=${!_export}"
    done

    # get new environment
    declare -A _env_new
    while IFS='=' read -r -d '' _name _value; do
        _env_new[$_name]="${_value@Q}"
    done < <(env -0)

    # determine what changed or became unset
    declare -a _env_changed_or_unset

    # changes
    for _name in "${!_env_new[@]}"; do
        if test "${_env_new[$_name]}" != "${_env_old[$_name]}"; then
            _env_changed_or_unset+=("$_name")
        fi
    done

    # unset
    for _name in "${!_env_old[@]}"; do
        if test ! -v "$_name"; then
            _env_changed_or_unset+=("$_name")
        fi
    done

    send_environment "$_call_id" "${_env_changed_or_unset[@]}" | jq -c
}

function bad_usage() {
    echo >&2 "usage: nu_plugin_bash_env [--stdin] [<env-file>]"
    echo >&2 "Maybe this plugin doesn't support your version of Nushell"
    echo >&2 "Please consider creating an issue at"
    echo >&2 "https://github.com/tesujimath/nu_plugin_bash_env/issues"
    test -n "$*" && echo >&2 "$*"
}

# process args
unset stdinval
declare -a exports
exports=()
unset path
while test -n "$1"; do
    case "$1" in
    --stdin)
        test -z "$stdinval" || {
            bad_usage "repeated --stdio"
            exit 1
        }
        stdinval="$(cat)"
        ;;
    --export)
        test "${#exports[@]}" -eq 0 || {
            bad_usage "repeated --export"
            exit 1
        }
        IFS=, read -r -a exports <<<"$2"
        shift
        ;;
    *)
        test -z "$path" || {
            bad_usage "repeated path"
            exit 1
        }
        path="$1"
        ;;
    esac
    shift
done

eval_and_or_source_then_send_environment "$stdinval" "$path" "${exports[@]}"
