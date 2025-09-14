use std::marker::PhantomData;

use super::*;


pub enum CoNever
{ }

impl<Ctx> CoroutineState<Ctx> for CoNever
{
	type Output = ();

	fn resume(self, _ctx: &mut Ctx) -> crate::CoResult<Self, Self::Output>
	{
		co_return(())
	}
}

pub struct CoNeverWithOutput<Output>
{
	_never: CoNever,
	_pd: PhantomData<fn() -> Output>,
}

impl<Ctx, Output> CoroutineState<Ctx> for CoNeverWithOutput<Output>
{
	type Output = Output;

	fn resume(self, _ctx: &mut Ctx) -> crate::CoResult<Self, Self::Output>
	{
		unreachable!("An instance of this type cannot be constructed")
	}
}
