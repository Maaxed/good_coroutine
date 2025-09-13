use super::*;

pub enum CoParallel<A, B>
{
	AB(A, B),
	A(A),
	B(B),
}

impl<A, B> CoParallel<A, B>
{
	pub(crate) fn new(a: A, b: B) -> Self
	{
		Self::AB(a, b)
	}
}

impl<Ctx, A, B> Coroutine<Ctx> for CoParallel<A, B>
where
	A: Coroutine<Ctx>,
	B: Coroutine<Ctx>,
{
	fn resume(self, ctx: &mut Ctx) -> CoResult<Self>
	{
		let (a, b) = match self
		{
			Self::AB(a, b) => (Some(a), Some(b)),
			Self::A(a) => (Some(a), None),
			Self::B(b) => (None, Some(b)),
		};

		let new_a = a.and_then(|a| a.resume(ctx).into());

		let new_b = b.and_then(|b| b.resume(ctx).into());

		match (new_a, new_b)
		{
			(None, None) => CoResult::Stop,
			(Some(a), Some(b)) => CoResult::RunNextFrame(Self::AB(a, b)),
			(Some(a), None) => CoResult::RunNextFrame(Self::A(a)),
			(None, Some(b)) => CoResult::RunNextFrame(Self::B(b)),
		}
	}
}
