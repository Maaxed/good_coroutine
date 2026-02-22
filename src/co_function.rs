use std::marker::PhantomData;

use super::*;

pub fn co_fn<Ctx, F, M>(f: F) -> CoFunction<F, F::State, M>
where
	F: CoFn<Ctx, M>
{
	CoFunction::Function(f, PhantomData)
}

pub enum CoFunction<F, S, M>
{
	Function(F, PhantomData<M>),
	State(S),
}

impl<Ctx, F, Output, C, M> Coroutine<Ctx> for CoFunction<F, F::State, M>
where
	C: Coroutine<Ctx, Output = Output>,
	F: CoFn<Ctx, M, Output = Output, State = C>,
{
	type Output = Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let res = match self
		{
			Self::Function(f, _) => f.co_call(ctx),
			Self::State(s) => s.resume(ctx),
		};

		match res
		{
			CoResult::Stop(res) => CoResult::Stop(res),
			CoResult::RunNextFrame(co) => CoResult::RunNextFrame(Self::State(co)),
		}
	}
}


pub trait CoFn<Ctx, M>
{
	type Output;
	type State;

	fn co_call(self, ctx: &mut Ctx) -> CoResult<Self::State, Self::Output>;
}

impl<Ctx, F, Output, C> CoFn<Ctx, fn(&mut Ctx) -> CoResult<C, Output>> for F
where
	F: FnOnce(&mut Ctx) -> CoResult<C, Output>
{
	type Output = Output;
	type State = C;

	fn co_call(self, ctx: &mut Ctx) -> CoResult<Self::State, Self::Output>
	{
		self(ctx)
	}
}

impl<Ctx, F, Output, C> CoFn<Ctx, fn() -> CoResult<C, Output>> for F
where
	F: FnOnce() -> CoResult<C, Output>
{
	type Output = Output;
	type State = C;

	fn co_call(self, _ctx: &mut Ctx) -> CoResult<Self::State, Self::Output>
	{
		self()
	}
}

impl<Ctx, F> CoFn<Ctx, fn(&mut Ctx)> for F
where
	F: FnOnce(&mut Ctx)
{
	type Output = ();
	type State = CoNever;

	fn co_call(self, ctx: &mut Ctx) -> CoResult<Self::State, Self::Output>
	{
		self(ctx);
		co_return(())
	}
}

impl<Ctx, F> CoFn<Ctx, fn()> for F
where
	F: FnOnce()
{
	type Output = ();
	type State = CoNever;

	fn co_call(self, _ctx: &mut Ctx) -> CoResult<Self::State, Self::Output>
	{
		self();
		co_return(())
	}
}
