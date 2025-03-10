# AI Snake Game

```bash
cargo build
cargo run
```

A TUI implementation of snake, built using Cursor IDE to try out coding without AI. Mostly usings Cursor's "Composer" feature. Dev process was roughly:

* Ask composer to do something
* Review the code
* Test it (mostly manually, i.e. play some snake)
* Ask composer to alter it and/or edit the code myself
* Commit it
* Repeat

You can see most of the prompts and responses in [./specstory/history](.specstory/history/2025-02-27_19-18-rust-terminal-snake-game-setup.md)

It felt very different to the normal process for coding a project, much more similar to the process of delegating a task to a co-worker. You have the requirements in your head and some opinions on how they should be implemented. The work needed to convey that to a co-worker feels different to coding directly. Much more up front thinking then normal for me, especially in a personal project.

I spent most of my time sitting thinking about what I wanted and reading code, a little time thinking about wording of prompts and almost no time typing.

For the code that got generated, it felt natural to treat it similarly to how I review PRs from actual people. The same trade offs apply - how thoroughly you want to review and understand the code depends on a bunch of factors.

TBD how well this works on more complicated projects, like spanning multiple repositories or infrastructure components, but it worked really well for this. I wrote almost no code myself, except for one ill fated attempt to be clever with how food was generated. Interesting next step would be to add a DB to save game state.

## Things that happened

Had a false start getting the project setup initially. [This issue](https://github.com/getcursor/cursor/issues/549) meant that my command to setup the project didn't work until I added `unset ARGV0 and then` to the start. That felt pretty similar to the kinds of random issues you always get setting up new dev environments for a project.

> unset ARGV0 and then initialise a cargo rust project in the current project folder for producing a terminal based game of snake using ratatui

If we're comparing Cursor to a human, Cursor's explanation of what its done and why are much more detailed then you normally get from a co-worker. In fact with most PRs I review I'd say writing up this level of detail in a commit message would be a waste of time. But given it's "free" here, it is really nice to have. [Specstory](https://specstory.com/) writes these to files and I should've added `.specstory/history` for every commit I made.

I can almost imagine that once you get used to using that to explain commits, you might prefer AI generated code to manually written code that doesn't come with such a detailed explanation. Or at least expect devs to generate explanations of their handwritten code. Also could see this encouraging me to code in a way that AI is more likely to understand, the same way you code so that your type system can understand.

When generating the algorithm for food placement, the AI first chose a naive algorithm that did:

* Choose random spot
* Is it already occupied by snake?
* If no - done, if yes, loop

I had an idea for a more sophisticated algorithm that would randomly choose from only valid spots first time. It was a struggle to convey this algorithm to the AI so I basically hand wrote it. Later when I realized my algorithm was buggy, the AI "fixed" it mostly by reverting to the initial algorithm. But not entirely - it did alter it slightly to avoid the possibility of looping forever which I assume was influenced by the fact my algorithm was trying to avoid that too?

Generating test was interesting, took a few goes to get the kind of coverage I wanted, but did save a lot of effort in boilerplate.