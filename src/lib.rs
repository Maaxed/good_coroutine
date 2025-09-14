mod co_function;
mod co_chain;
mod co_never;
mod co_box;
mod co_loop;
mod co_concurrent;

pub use co_function::*;
pub use co_chain::*;
pub use co_never::*;
pub use co_box::*;
pub use co_loop::*;
pub use co_concurrent::*;

pub mod prelude
{
	pub use crate::{
		Coroutine,
		CoResult,
		co_return,
		co_yield,
		co_fn,
		co_chain,
		co_concurrent,
	};
}

pub trait Coroutine<Ctx, Input>: Sized
{
	type Output;
	type State: CoroutineState<Ctx, Output = Self::Output>;

	fn init(self, ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>;
}

#[must_use]
pub trait CoroutineState<Ctx>: Sized
{
	type Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>;
}


#[must_use]
pub enum CoResult<Co, Output>
{
	Stop(Output),
	RunNextFrame(Co),
}


pub fn co_return<Co, Output>(res: Output) -> CoResult<Co, Output>
{
	CoResult::Stop(res)
}

pub fn co_yield<Ctx, C>(coroutine: C, ctx: &mut Ctx) -> CoResult<C::State, C::Output>
where
	C: Coroutine<Ctx, ()>,
{
	coroutine.init(ctx, ())
}

pub struct CoNextFrame(bool);

impl<Ctx> Coroutine<Ctx, ()> for CoNextFrame
{
	type Output = ();
	type State = Self;

	fn init(self, ctx: &mut Ctx, _i: ()) -> CoResult<Self::State, Self::Output>
	{
		self.resume(ctx)
	}
}

impl<Ctx> CoroutineState<Ctx> for CoNextFrame
{
	type Output = ();

	fn resume(self, _ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		if self.0
		{
			co_return(())
		}
		else
		{
			CoResult::RunNextFrame(Self(true))
		}
	}
}

pub fn co_next_frame() -> CoNextFrame
{
	CoNextFrame(false)
}


pub struct IgnoreOutput<T>(T);

impl<Ctx, T, Input> Coroutine<Ctx, Input> for IgnoreOutput<T>
	where
		T: Coroutine<Ctx, Input>
{
	type Output = ();
	type State = IgnoreOutput<T::State>;

	fn init(self, ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>
	{
		match self.0.init(ctx, input)
		{
			CoResult::Stop(_res) => CoResult::Stop(()),
			CoResult::RunNextFrame(co) => CoResult::RunNextFrame(IgnoreOutput(co))
		}
	}
}

impl<Ctx, T> CoroutineState<Ctx> for IgnoreOutput<T>
	where
		T: CoroutineState<Ctx>
{
	type Output = ();

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		match self.0.resume(ctx)
		{
			CoResult::Stop(_res) => CoResult::Stop(()),
			CoResult::RunNextFrame(co) => CoResult::RunNextFrame(IgnoreOutput(co))
		}
	}
}


#[cfg(test)]
mod tests
{
	use crate::*;

	#[test]
	fn basic_coroutine()
	{
		fn coroutine_function() -> impl Coroutine<Vec<u32>, (), Output = ()>
		{
			co_fn(|ctx: &mut Vec<u32>|
			{
				let mut vec = Vec::new();

				for i in 0..4
				{
					vec.push(co_fn(move |ctx: &mut Vec<u32>|
					{
						ctx.push(i);
					}));
				}

				co_yield(co_chain(vec), ctx)
			})
		}

		let coroutine = coroutine_function();

		let mut ctx = Vec::new();
		let res = coroutine.init(&mut ctx, ());
		assert!(matches!(res, CoResult::Stop(())));

		assert_eq!(ctx, vec![0, 1, 2, 3]);
	}

	#[test]
	fn coroutine_next_frame()
	{
		fn coroutine_function2() -> impl Coroutine<Vec<u32>, (), Output = ()>
		{
			co_fn(|ctx: &mut Vec<u32>|
			{
				let mut vec = Vec::new();

				for i in 0..2
				{
					vec.push(co_fn(move |ctx: &mut Vec<u32>|
					{
						ctx.push(i);

						co_yield(co_next_frame(), ctx)
					}));
				}

				co_yield(co_chain(vec), ctx)
			})
		}

		let coroutine = coroutine_function2();
		let mut ctx = Vec::new();

		let res = coroutine.init(&mut ctx, ());
		let CoResult::RunNextFrame(coroutine) = res
		else
		{
			panic!("assertion failed: expected CoResult::RunNextFrame");
		};
		assert_eq!(ctx, vec![0]);

		let res = coroutine.resume(&mut ctx);
		let CoResult::RunNextFrame(coroutine) = res
		else
		{
			panic!("assertion failed: expected CoResult::RunNextFrame");
		};
		assert_eq!(ctx, vec![0, 1]);

		let res = coroutine.resume(&mut ctx);
		assert!(matches!(res, CoResult::Stop(())));
		assert_eq!(ctx, vec![0, 1]);
	}
}
