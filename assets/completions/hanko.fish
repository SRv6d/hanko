# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_hanko_global_optspecs
	string join \n c/config= file= v/verbose h/help V/version
end

function __fish_hanko_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_hanko_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_hanko_using_subcommand
	set -l cmd (__fish_hanko_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c hanko -n "__fish_hanko_needs_command" -s c -l config -d 'The configuration file' -r -F
complete -c hanko -n "__fish_hanko_needs_command" -l file -d 'The allowed signers file' -r -F
complete -c hanko -n "__fish_hanko_needs_command" -s v -l verbose -d 'Use verbose output'
complete -c hanko -n "__fish_hanko_needs_command" -s h -l help -d 'Print help'
complete -c hanko -n "__fish_hanko_needs_command" -s V -l version -d 'Print version'
complete -c hanko -n "__fish_hanko_needs_command" -f -a "update" -d 'Update the allowed signers file'
complete -c hanko -n "__fish_hanko_needs_command" -f -a "signer" -d 'Manage allowed signers'
complete -c hanko -n "__fish_hanko_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hanko -n "__fish_hanko_using_subcommand update" -s c -l config -d 'The configuration file' -r -F
complete -c hanko -n "__fish_hanko_using_subcommand update" -l file -d 'The allowed signers file' -r -F
complete -c hanko -n "__fish_hanko_using_subcommand update" -s v -l verbose -d 'Use verbose output'
complete -c hanko -n "__fish_hanko_using_subcommand update" -s h -l help -d 'Print help'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and not __fish_seen_subcommand_from add help" -s c -l config -d 'The configuration file' -r -F
complete -c hanko -n "__fish_hanko_using_subcommand signer; and not __fish_seen_subcommand_from add help" -l file -d 'The allowed signers file' -r -F
complete -c hanko -n "__fish_hanko_using_subcommand signer; and not __fish_seen_subcommand_from add help" -s v -l verbose -d 'Use verbose output'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and not __fish_seen_subcommand_from add help" -s h -l help -d 'Print help'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and not __fish_seen_subcommand_from add help" -f -a "add" -d 'Add an allowed signer'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and not __fish_seen_subcommand_from add help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from add" -s s -l source -d 'The source(s) of the signer to add' -r
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from add" -s c -l config -d 'The configuration file' -r -F
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from add" -l file -d 'The allowed signers file' -r -F
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from add" -l no-update -d 'Don\'t update the allowed signers file with the added signer(s)'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Use verbose output'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add an allowed signer'
complete -c hanko -n "__fish_hanko_using_subcommand signer; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hanko -n "__fish_hanko_using_subcommand help; and not __fish_seen_subcommand_from update signer help" -f -a "update" -d 'Update the allowed signers file'
complete -c hanko -n "__fish_hanko_using_subcommand help; and not __fish_seen_subcommand_from update signer help" -f -a "signer" -d 'Manage allowed signers'
complete -c hanko -n "__fish_hanko_using_subcommand help; and not __fish_seen_subcommand_from update signer help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hanko -n "__fish_hanko_using_subcommand help; and __fish_seen_subcommand_from signer" -f -a "add" -d 'Add an allowed signer'
