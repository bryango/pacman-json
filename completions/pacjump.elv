
use builtin;
use str;

set edit:completion:arg-completer[pacjump] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'pacjump'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'pacjump'= {
            cand --recurse 'Recursively query the dependencies of the given package; implies ''--all'''
            cand --sync 'Query the sync databases; by default only the local database (of currently installed packages) is queried'
            cand --all 'Query all packages, including those not explicitly installed; by default only explicitly installed packages are shown'
            cand --plain 'Output package info from the current database only; by default we enrich the output by combining information from both the local and the sync databases'
            cand --optional '''--recurse'' optional dependencies as well'
            cand --summary '''--recurse'' dependencies, but only prints package names and versions'
            cand -h 'Print help'
            cand --help 'Print help'
        }
    ]
    $completions[$command]
}
