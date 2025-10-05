
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'hanko' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'hanko'
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
        'hanko' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--file', '--file', [CompletionResultType]::ParameterName, 'The allowed signers file')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update the allowed signers file')
            [CompletionResult]::new('signer', 'signer', [CompletionResultType]::ParameterValue, 'Manage allowed signers')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hanko;update' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--file', '--file', [CompletionResultType]::ParameterName, 'The allowed signers file')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hanko;signer' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--file', '--file', [CompletionResultType]::ParameterName, 'The allowed signers file')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add an allowed signer')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hanko;signer;add' {
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'The source(s) of the signer to add')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'The source(s) of the signer to add')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'The configuration file')
            [CompletionResult]::new('--file', '--file', [CompletionResultType]::ParameterName, 'The allowed signers file')
            [CompletionResult]::new('--no-update', '--no-update', [CompletionResultType]::ParameterName, 'Don''t update the allowed signers file with the added signer(s)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hanko;signer;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add an allowed signer')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hanko;signer;help;add' {
            break
        }
        'hanko;signer;help;help' {
            break
        }
        'hanko;help' {
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update the allowed signers file')
            [CompletionResult]::new('signer', 'signer', [CompletionResultType]::ParameterValue, 'Manage allowed signers')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hanko;help;update' {
            break
        }
        'hanko;help;signer' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add an allowed signer')
            break
        }
        'hanko;help;signer;add' {
            break
        }
        'hanko;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
