use std::marker::PhantomData;

use super::*;

pub fn co_fn<Ctx, F, Input, M>(f: F) -> CoFunction<F, M>
where
	F: CoFn<Ctx, Input, M>
{
	CoFunction(f, PhantomData)
}

pub struct CoFunction<F, M>(F, PhantomData<M>);

impl<Ctx, F, Input, Output, C, M> Coroutine<Ctx, Input> for CoFunction<F, M>
where
	C: CoroutineState<Ctx, Output = Output>,
	F: CoFn<Ctx, Input, M, Output = Output, State = C>,
{
	type Output = Output;
	type State = C;

	fn init(self, ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>
	{
		self.0.co_call(ctx, input)
	}
}


pub trait CoFn<Ctx, Input, M>
{
	type Output;
	type State;

	fn co_call(self, ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>;
}

impl<Ctx, F, Input, Output, C> CoFn<Ctx, Input, fn(&mut Ctx, Input) -> CoResult<C, Output>> for F
where
	F: FnOnce(&mut Ctx, Input) -> CoResult<C, Output>
{
	type Output = Output;
	type State = C;

	fn co_call(self, ctx: &mut Ctx, input: Input) -> CoResult<Self::State, Self::Output>
	{
		self(ctx, input)
	}
}

impl<Ctx, F, Output, C> CoFn<Ctx, (), fn(&mut Ctx) -> CoResult<C, Output>> for F
where
	F: FnOnce(&mut Ctx) -> CoResult<C, Output>
{
	type Output = Output;
	type State = C;

	fn co_call(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
	{
		self(ctx)
	}
}

impl<Ctx, F, Output, C> CoFn<Ctx, (), fn() -> CoResult<C, Output>> for F
where
	F: FnOnce() -> CoResult<C, Output>
{
	type Output = Output;
	type State = C;

	fn co_call(self, _ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
	{
		self()
	}
}

impl<Ctx, F> CoFn<Ctx, (), fn(&mut Ctx)> for F
where
	F: FnOnce(&mut Ctx)
{
	type Output = ();
	type State = CoNever;

	fn co_call(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
	{
		self(ctx);
		co_return(())
	}
}

impl<Ctx, F> CoFn<Ctx, (), fn()> for F
where
	F: FnOnce()
{
	type Output = ();
	type State = CoNever;

	fn co_call(self, _ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
	{
		self();
		co_return(())
	}
}
