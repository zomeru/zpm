function _zpm_completion
  set -l tokens (commandline -xpc)
  if test (count $tokens) -ge 1
    set tokens $tokens[2..-1]
  end
  zpm completion --fish $tokens 2>/dev/null
end
complete -c zpm -f -a '(_zpm_completion)' -d 'zpm commands'
