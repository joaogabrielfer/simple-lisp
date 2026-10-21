-- Neovim plugin for evaluating expressions
-- for using it, highlight the expression and use the editor command `LispEval`

local function lisp_eval()
  local s_pos = vim.fn.getpos("'<")
  local e_pos = vim.fn.getpos("'>")

  local lines = vim.fn.getregion(s_pos, e_pos, { type = vim.fn.visualmode() })

  local code = table.concat(lines, "\n")

  local obj = vim.system({ "cargo", "run", "--", "eval", code }):wait()

  if obj.code == 0 then
    print(obj.stdout)
  else
    vim.notify("Error: " .. obj.stderr, vim.log.levels.ERROR)
  end
end

vim.api.nvim_create_user_command("LispEval", lisp_eval, {
  desc = "eval current highlighted code",
  range = true,
})
