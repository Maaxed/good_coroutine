use super::*;


#[must_use]
pub fn co_loop<C>(c: C) -> CoLoop<C>
{
	CoLoop(c)
}

/// Rerun the current coroutine next update.
pub fn co_continue() -> CoLoopResult
{
	CoLoopResult::Continue
}

/// Stops the execution of the current coroutine.
pub fn co_break() -> CoLoopResult
{
	CoLoopResult::Break
}


pub struct CoLoop<F>(F);

impl<Ctx, F> Coroutine<Ctx> for CoLoop<F>
where
	F: FnMut(&mut Ctx) -> CoLoopResult,
{
	fn resume(mut self, ctx: &mut Ctx) -> CoResult<Self>
	{
		let res = (self.0)(ctx);

		match res
		{
			CoLoopResult::Break => CoResult::Stop,
			CoLoopResult::Continue => CoResult::RunNextFrame(self),
		}
	}
}


/// The result of a coroutine loop resumption.
#[must_use]
pub enum CoLoopResult
{
    Continue,
    Break,
}
