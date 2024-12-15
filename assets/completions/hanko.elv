
use builtin;
use str;

set edit:completion:arg-completer[hanko] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'hanko'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'hanko'= {
            cand -c 'The configuration file'
            cand --config 'The configuration file'
            cand --file 'The allowed signers file'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand update 'Update the allowed signers file'
            cand signer 'Manage allowed signers'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hanko;update'= {
            cand -c 'The configuration file'
            cand --config 'The configuration file'
            cand --file 'The allowed signers file'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hanko;signer'= {
            cand -c 'The configuration file'
            cand --config 'The configuration file'
            cand --file 'The allowed signers file'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand add 'Add an allowed signer'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hanko;signer;add'= {
            cand -s 'The source(s) of the signer to add'
            cand --source 'The source(s) of the signer to add'
            cand -c 'The configuration file'
            cand --config 'The configuration file'
            cand --file 'The allowed signers file'
            cand --no-update 'Don''t update the allowed signers file with the added signer(s)'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hanko;signer;help'= {
            cand add 'Add an allowed signer'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hanko;signer;help;add'= {
        }
        &'hanko;signer;help;help'= {
        }
        &'hanko;help'= {
            cand update 'Update the allowed signers file'
            cand signer 'Manage allowed signers'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hanko;help;update'= {
        }
        &'hanko;help;signer'= {
            cand add 'Add an allowed signer'
        }
        &'hanko;help;signer;add'= {
        }
        &'hanko;help;help'= {
        }
    ]
    $completions[$command]
}
