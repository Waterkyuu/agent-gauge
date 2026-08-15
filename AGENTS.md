# AGENTS.md

## Git 提交规则

- 每次任务完成后主动提交代码。
- Git 提交信息必须使用英文，并采用 `feat`、`fix`、`docs`、`chore`、`style`、`ci` 等规范前缀。
- 每个提交只包含一个业务逻辑，最多提交 6 个文件，不得混入无关修改。
- 提交信息使用以下格式：

```text
feat(scope): summary
- change one
- change two
- change three
```

- 任何被 `.gitignore` 匹配的文件或目录都禁止擅自加入 Git 或提交。
- 禁止使用 `git add -f`、`git add --force` 或其他方式绕过 `.gitignore`。
- 只有用户明确指定某个被忽略的路径需要提交时，才允许强制加入该路径。
- 提交前必须检查待提交文件，确保没有误提交被忽略的文件、用户的其他修改或与当前任务无关的内容。
