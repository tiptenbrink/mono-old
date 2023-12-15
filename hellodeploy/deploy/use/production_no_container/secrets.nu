#!/usr/bin/env nu
def secret_to_pipe [secret: string] {
    # we request the secret from Bitwarden Secret Manager using its id, loading it as JSON
    let bws_res = do { bws secret get $secret } | complete
    let bws_exit = $bws_res | get exit_code

    if $bws_exit != 0 {
        print $"Bitwarden Secrets Manager CLI failed with output:\n ($bws_res | get stderr)"
        exit $bws_exit
    }

    let j = $bws_res | get stdout | from json

    let k = $j | get key
    let v = $j | get value
    return $"($k)=($v)"
}

def from_deploy [file] {
    let j = open $file --raw | decode utf-8 | from json
    let secrets = $j | get secrets
    let output = $secrets | par-each { |e| secret_to_pipe $e } | str join "\n"
    print $output
}

# main entrypoint
def main [] {
    if ('secrets.json.local' | path exists) {
        from_deploy secrets.json.local
    } else {
        from_deploy secrets.json
    }
    
}