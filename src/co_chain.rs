use super::*;

pub struct CoChain<A, B>(Option<A>, B);

impl<A, B> CoChain<A, B>
{
	pub(crate) fn new(a: A, b: B) -> Self
	{
		Self(Some(a), b)
	}
}

impl<Ctx, A, B> Coroutine<Ctx> for CoChain<A, B>
where
	A: Coroutine<Ctx>,
	B: Coroutine<Ctx>,
{
	fn resume(self, ctx: &mut Ctx) -> CoResult<Self>
	{
		let Self(a, b) = self;

		if let Some(a) = a
		{
			match a.resume(ctx)
			{
				CoResult::RunNextFrame(co) => return CoResult::RunNextFrame(Self(Some(co), b)),
				CoResult::Stop => { },
			}
		}

		match b.resume(ctx)
		{
			CoResult::RunNextFrame(co) => CoResult::RunNextFrame(Self(None, co)),
			CoResult::Stop => CoResult::Stop,
		}
	}
}
