# good_coroutine

A library for running coroutines in rust designed for games  
**This library is very experimental, expect breaking changes!**

## Goal

The main motivation behind this library is to allow you to spread the execution of a function across several frames.
It also helps with managing and running functions in sequence over time.
good_coroutine is meant for concurrent execution of coroutines but not for parallel execution.

## Creating coroutines

A coroutine can be created from a function or from a closure using the ``co_fn`` function.

Coroutine functions don't use a 'yield' statement but instead return a ``CoResult``.
The ``CoResult`` indicates if the coroutine should stop and return a result or continue running at the next update.
In addition, a coroutine can run other coroutines and wait for their completion by calling and returning ``co_await``.

```rust
use good_coroutine::prelude::*;

fn coroutine_function() -> impl Coroutine<(), Output = ()>
{
	co_fn(|ctx: &mut ()|
	{
		let mut vec = Vec::new();

		for i in 0..4
		{
			// Wait next frame then print the number i in the console
			vec.push(co_chain((
				co_next_frame(),
				move |()| print_number(i),
			)));
		}

		co_await(co_chain(vec), ctx)
	})
}

fn print_number(
	i: u32,
) -> impl Coroutine<(), Output = ()>
{
	co_fn(move |_ctx: &mut ()|
	{
		println!("{i}");
	})
}
```

Coroutines can be combined in the following ways:
- The ``co_chain`` function can be used to chain two or more coroutines returning a new coroutine that executes each coroutine one by one launching the next when the previous completes
- The ``co_concurrent`` function can be used to run coroutines concurrently. It returns a new coroutine that executes all coroutines at the same time and completes when all of the coroutines are completed
