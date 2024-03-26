# VS code setup

If you cannot use RustRover, then use vscode for `tournament-organiser-web`.
Install [VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (
and
disable Vetur)

This project uses eslint, use eslint vscode extension with the following editor
settings:

```json
    ...
"editor.rulers": [80],
"editor.codeActionsOnSave": {
"source.fixAll": "never",
"source.fixAll.eslint": "always"
},
"eslint.validate": [
"javascript"
],
"todohighlight.keywords": [
{
"text": "TODO",
"color": "rgb(13, 184, 38)",
"backgroundColor": "rgba(6, 89, 18,.4)",
"isWholeLine": true,
},
{
"text": "FIXME",
"color": "rgb(242, 51, 51)",
"backgroundColor": "rgba(138, 28, 28,.4)",
"isWholeLine": true,
},
{
"text": "NOTE",
"color": "rgb(13, 184, 38)",
"backgroundColor": "rgba(6, 89, 18,.1)",
"isWholeLine": false,
},
],
...
```
