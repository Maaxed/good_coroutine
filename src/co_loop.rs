use super::*;


#[must_use]
pub fn co_loop<C>(c: C) -> CoLoop<C>
{
	CoLoop(c)
}

/// Rerun the current coroutine next update.
pub fn co_continue<Output>() -> CoLoopResult<Output>
{
	CoLoopResult::Continue
}

/// Stops the execution of the current coroutine.
pub fn co_break<Output>(output: Output) -> CoLoopResult<Output>
{
	CoLoopResult::Break(output)
}


pub struct CoLoop<F>(F);

impl<Ctx, F, Output> Coroutine<Ctx> for CoLoop<F>
where
	F: FnMut(&mut Ctx) -> CoLoopResult<Output>,
{
	type Output = Output;
	fn resume(mut self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let res = (self.0)(ctx);

		match res
		{
			CoLoopResult::Break(res) => CoResult::Stop(res),
			CoLoopResult::Continue => CoResult::RunNextFrame(self),
		}
	}
}


/// The result of a coroutine loop resumption.
#[must_use]
pub enum CoLoopResult<Output>
{
	Continue,
	Break(Output),
}
