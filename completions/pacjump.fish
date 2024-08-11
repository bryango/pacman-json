complete -c pacjump -l recurse -d 'Recursively query the dependencies of the given package; implies \'--all\'' -r
complete -c pacjump -l sync -d 'Query the sync databases; by default only the local database (of currently installed packages) is queried'
complete -c pacjump -l all -d 'Query all packages, including those not explicitly installed; by default only explicitly installed packages are shown'
complete -c pacjump -l plain -d 'Output package info from the current database only; by default we enrich the output by combining information from both the local and the sync databases'
complete -c pacjump -l optional -d '\'--recurse\' optional dependencies as well'
complete -c pacjump -l summary -d '\'--recurse\' dependencies, but only prints package names and versions'
complete -c pacjump -s h -l help -d 'Print help'
