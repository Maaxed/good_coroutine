use super::*;
use std::collections::VecDeque;

pub fn co_vec<Co>() -> CoVec<Co>
{
	CoVec(VecDeque::new())
}

pub struct CoVec<Co>(VecDeque<Co>);

impl<Co> CoVec<Co>
{
	pub fn push<Ctx, M>(&mut self, co: impl IntoCoroutine<Ctx, M, Coroutine = Co>)
	{
		self.0.push_back(co.into_coroutine());
	}
}

impl<Ctx, Co> Coroutine<Ctx> for CoVec<Co>
where
	Co: Coroutine<Ctx>,
{
	fn resume(mut self, ctx: &mut Ctx) -> CoResult<Self>
	{
		while let Some(co) = self.0.pop_front()
		{
			if let CoResult::RunNextFrame(co) = co.resume(ctx)
			{
				self.0.push_front(co);
				return CoResult::RunNextFrame(self)
			}
		}

		CoResult::Stop
	}
}
