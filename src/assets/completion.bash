###-begin-zpm-completion-###
if type complete &>/dev/null; then
  _zpm_completion() {
    local words
    local cur
    local cword
    _get_comp_words_by_ref -n =: cur words cword
    IFS=$'\n'
    COMPREPLY=($(COMP_CWORD=$cword COMP_LINE=$cur zpm completion --bash ${words[@]}))
  }
  complete -F _zpm_completion zpm
fi
###-end-zpm-completion-###
