use super::*;

pub fn co_fn<Ctx, F, C>(f: F) -> CoFunction<F, C>
where
	F: FnOnce(&mut Ctx) -> CoResult<C>
{
	CoFunction::Function(f)
}

pub enum CoFunction<F, C>
{
	Function(F),
	Result(C),
}

impl<Ctx, F, C> Coroutine<Ctx> for CoFunction<F, C>
where
	C: Coroutine<Ctx>,
	F: FnOnce(&mut Ctx) -> CoResult<C>,
{
	fn resume(self, ctx: &mut Ctx) -> CoResult<Self>
	{
		let res = match self
		{
			CoFunction::Function(f) => f(ctx),
			CoFunction::Result(r) => r.resume(ctx),
		};
		match res
		{
			CoResult::Stop => CoResult::Stop,
			CoResult::RunNextFrame(co) => CoResult::RunNextFrame(Self::Result(co)),
		}
	}
}
