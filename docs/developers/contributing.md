# Contributing

The easiest way to contribute to the Sabre project is by writing code in the language. The more the language is battle-tested, the more bugs can be found and therefore the language becomes more stable.

## Getting in touch

If you have a question, found a potential bug or want to engage with the community, you can join the [matrix room](https://matrix.to/#/#sabre:matrix.slashdev.space?via=matrix.slashdev.space) of this project. If you prefer to stay away from matrix, you can also send a mail (or a patch) to the [public mailing list](https://lists.sr.ht/~garritfra/sabre).

## Fixing things and adding features

If you want to contribute to the compiler itself, the easiest way to get started is to look at the [TODO file](https://github.com/garritfra/sabre/blob/master/TODO) at the root of the project. Usually, this is where important todo items are jotted down.

You could also run the tests (`cargo test`) and see if any tests are ignored. Usually, if a bug is found in the wild, a failing but ignored test is written, so that it can be further investigated later.

## Writing documentation

As with all software, Sabre needs good documentation. Since Sabre is still in early development, things change constantly. This means that docs will be out of date in a lot of cases, or not written at all. Any help with the documentation is greatly appreciated!

## Submitting your code

If you want to contribute code, please open a pull request on [GitHub](https://github.com/garritfra/sabre). There is also a [SourceHut mirror](https://sr.ht/~garritfra/sabre/), if you're trying to avoid GitHub. Feel free to send a patch to the [public mailing list](https://lists.sr.ht/~garritfra/sabre). Check out [this guide](https://slashdev.space/posts/patch-based-git-workflow) to learn about the patch based workflow.

Before submitting the code, please make sure that it is **sufficiently documented and tested**.
