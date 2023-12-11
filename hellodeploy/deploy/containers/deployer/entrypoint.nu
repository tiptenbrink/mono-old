#!/usr/bin/env nu
def secret_to_pipe [secret: string, pipe: string] {
    # we request the secret from Bitwarden Secret Manager using its id, loading it as JSON
    let j = bws secret get $secret | from json
    # the key is identical to the environment variable key
    let k = $j | get key
    # the value is the actual secret
    let v = $j | get value
    # for each secret we append a newline and <KEY>=<VALUE> to the named pipe
    echo $"\n($k)=($v)" | save $pipe --append
}

def from_deploy [file, pipe: string] {
    # open the file and load it as Nu's JSON representation
    let j = open $file --raw | decode utf-8 | from json
    # we get the value of secrets.ids, which is an array of id values
    let secrets = $j | get secrets | get ids
    # we call the secret_to_pipe function for each id and also add the name of the pipe as an argument
    for $e in $secrets { secret_to_pipe $e $pipe }
}

# main entrypoint
def main [] {
    # we call the from_deploy function with arguments tidploy.json and ti_dploy_pipe
    from_deploy tidploy.json ti_dploy_pipe
    # once we're done we append a newline and TIDPLOY_READY=1 to the named pipe
    echo "\nTIDPLOY_READY=1" | save ti_dploy_pipe --append
}