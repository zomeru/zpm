#compdef zpm
_zpm_completion() {
  local -a completions
  completions=("${(f)$(zpm completion --zsh $words[2,-1])}")
  compadd -a completions
}
_zpm_completion
