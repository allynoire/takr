# Takr - Task Tracker

Light weight file based task tracker inspired by [Tatr](https://github.com/tsoding/tatr).

Takr operates on the `tasks/` directory of the current folder. It will try to interpret any markdown file in your task directory as a task. Task files must define the following frontmatter properties otherwise Takr will ignore them. All required properties must be single lines. Json syntax should be used when defining tags.

```md
---
title: buy coffee
status: open
rank: 100
tags: [ coffee, groceries ]
# whatever other properties you need...
---
# whatever you want to put here :)
```

You may use the body of the file as needed and define additional frontmatter fields if required.

## Usage

### Add task

Takr will create new task files for you with the `add` command.

```shell
$ takr add <TITLE> -r <RANK> -t <TAGS>... 
```

### List tasks

To list existing tasks use the `list` command.

```shell
$ takr list

<tasks\260914-153710.md>   161   ABC.............................   (a) (b) (c)
<tasks\260914-153646.md>    90   AB..............................   (a) (b)
<tasks\260914-153651.md>    80   BC..............................   (b) (c)
<tasks\260914-153655.md>    70   CA..............................   (a) (c)
<tasks\260914-153557.md>    20   A...............................   (a)
<tasks\260914-153601.md>    13   B...............................   (b)
<tasks\260914-153607.md>    12   C...............................   (c)
<tasks\260914-154649.md>     0   untagged........................
```

You may specify the following flags to filter tasks based on their tags or content. You may specify the flags `-t` and `-T` multiple times.

- `-f` Text filter.
- `-a` Include closed tasks.
- `-u` Exclude tagged.
- `-U` Exclude untagged.
- `-t <TAG>...` Include tasks with specified tags.
- `-T <TAG>...` Exclude tasks with specified tags.

#### Query Examples

| Query         | Lists tasks that                             |
| ------------- | -------------------------------------------- |
| `-f test`     | contain the substring `"test"` in the title  |
| `-u`          | have no tags                                 |
| `-U`          | have tags                                    |
| `-t a`        | contain the tag `a`                          |
| `-t a b`      | contain the tag `a` or `b`                   |
| `-t a -t b`   | contain the tag `a` and `b`                  |
| `-t a -t b c` | contain the tag `a` and (`b` or `c`)         |
| `-T a`        | don't contain the tag `a`                    |
| `-T a b`      | don't contain the tag (`a` or `b`)           |
| `-T a -T b`   | don't contain the tag (`a` and `b`)          |
| `-T a -T b c` | don't contain the tag (`a` and (`b` or `c`)) |
