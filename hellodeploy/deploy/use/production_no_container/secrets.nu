#!/usr/bin/env nu
def secret_to_pipe [secret: string] {
    # we request the secret from Bitwarden Secret Manager using its id, loading it as JSON
    let j = bws secret get $secret | from json
    let k = $j | get key
    let v = $j | get value
    return $"($k)=($v)"
}

def from_deploy [file] {
    let j = open $file --raw | decode utf-8 | from json
    let secrets = $j | get secrets | get ids
    let output = $secrets | par-each { |e| secret_to_pipe $e } | str join "\n"
    print $output
}

# main entrypoint
def main [] {
    from_deploy tidploy.json
}