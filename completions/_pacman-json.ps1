
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'pacman-json' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'pacman-json'
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
        'pacman-json' {
            [CompletionResult]::new('--sync', 'sync', [CompletionResultType]::ParameterName, 'Query the sync databases; by default we only query the local database with the currently installed packages')
            [CompletionResult]::new('--all', 'all', [CompletionResultType]::ParameterName, 'Query all packages, including those not explicitly installed; by default only explicitly installed packages are shown')
            [CompletionResult]::new('--plain', 'plain', [CompletionResultType]::ParameterName, 'Output package info from the current database only; by default we enrich the output by combining information from both the local and the sync databases')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
