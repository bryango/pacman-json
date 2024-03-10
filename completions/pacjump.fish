complete -c pacjump -l sync -d 'Query the sync databases; by default we only query the local database with the currently installed packages'
complete -c pacjump -l all -d 'Query all packages, including those not explicitly installed; by default only explicitly installed packages are shown'
complete -c pacjump -l plain -d 'Output package info from the current database only; by default we enrich the output by combining information from both the local and the sync databases'
complete -c pacjump -s h -l help -d 'Print help'
