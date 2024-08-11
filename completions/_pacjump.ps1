
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'pacjump' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'pacjump'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'pacjump' {
            [CompletionResult]::new('--recurse', '--recurse', [CompletionResultType]::ParameterName, 'Recursively query the dependencies of the given package; implies ''--all''')
            [CompletionResult]::new('--sync', '--sync', [CompletionResultType]::ParameterName, 'Query the sync databases; by default only the local database (of currently installed packages) is queried')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Query all packages, including those not explicitly installed; by default only explicitly installed packages are shown')
            [CompletionResult]::new('--plain', '--plain', [CompletionResultType]::ParameterName, 'Output package info from the current database only; by default we enrich the output by combining information from both the local and the sync databases')
            [CompletionResult]::new('--optional', '--optional', [CompletionResultType]::ParameterName, '''--recurse'' optional dependencies as well')
            [CompletionResult]::new('--summary', '--summary', [CompletionResultType]::ParameterName, '''--recurse'' dependencies, but only prints package names and versions')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
